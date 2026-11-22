//! Rulings batch P079 — triggers on attacks against the defending player: "whenever a
//! creature attacks you" (once per creature, only declared attackers, CR 508.3a, 508.4),
//! "whenever one or more / two or more creatures attack" (CR 508.3d), "whenever a player
//! attacks you" (CR 508.3c), intervening-if clauses (CR 603.4) and counting attackers as
//! the ability resolves.

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s07_common::damage_on;
use crate::r_s10_common::attacking;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn blood_reckoning_triggers_once_per_declared_attacker() {
    cr!("508.3a", "508.4", "603.2");
    ruling!(
        "Blood Reckoning",
        "Blood Reckoning will trigger once for each creature that attacks you or a planeswalker you control."
    );
    ruling!(
        "Blood Reckoning",
        "Creatures that are put onto the battlefield attacking were never declared as attackers"
    );
    supported("Blood Reckoning");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Blood Reckoning");
    let pw = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Savannah Lions");
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, obj(pw)),
        ],
    );
    t.resolve_all();
    assert_eq!(
        t.life(P0),
        17,
        "one trigger per creature, the planeswalker's included"
    );
    // A creature put onto the battlefield attacking P1 doesn't trigger it.
    let new = enter_attacking(&mut t, P0, "Grizzly Bears", Entity::Player(P1));
    assert!(attacking(&t, new));
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn circle_of_flame_checks_flying_as_attackers_are_declared() {
    cr!("508.3a", "508.4", "603.2", "603.10");
    ruling!(
        "Circle of Flame",
        "Circle of Flame's ability triggers only if a creature without flying is declared as an attacker during the declare attackers step."
    );
    supported("Circle of Flame");
    supported("Gravity Well");
    // A flyer attacks P1 and loses flying to Gravity Well: Circle of Flame didn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Circle of Flame");
    t.battlefield(P1, "Gravity Well");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(angel, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(
        !t.obj_now(angel)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying),
        "Gravity Well removed its flying"
    );
    assert_eq!(
        damage_on(&t, angel),
        0,
        "no trigger for the creature that attacked flying"
    );
    assert_eq!(
        damage_on(&t, bears),
        1,
        "the creature without flying was dealt 1"
    );
    // A creature without flying put onto the battlefield attacking doesn't trigger it.
    let new = enter_attacking(&mut t, P0, "Hill Giant", Entity::Player(P1));
    t.resolve_all();
    assert_eq!(damage_on(&t, new), 0);
}

#[test]
fn flying_granted_after_gravity_well_resolves_works() {
    cr!("508.3a", "603.3b", "611.2c");
    ruling!(
        "Gravity Well",
        "After attackers are declared and any Gravity Well abilities have resolved, a spell or ability that grants flying to a creature will work normally."
    );
    supported("Gravity Well");
    supported("Order of the Golden Cricket");
    supported("Jump");
    // P0's Order of the Golden Cricket and Serra Angel attack P1, who controls Gravity
    // Well. P0's Cricket trigger goes on the stack first (APNAP), so Gravity Well's
    // triggers resolve first; the Cricket then gains flying, and Jump gives the Angel
    // flying back.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Gravity Well");
    let cricket = t.battlefield(P0, "Order of the Golden Cricket");
    let angel = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[(cricket, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    t.resolve_all();
    let flying = |t: &TestGame, id| {
        t.obj_now(id)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying)
    };
    assert!(
        flying(&t, cricket),
        "the Cricket's own attack trigger resolved last"
    );
    assert!(!flying(&t, angel));
    cast_card(&mut t, P0, "Jump", &[obj(angel)]).unwrap();
    t.resolve_all();
    assert!(flying(&t, angel));
}

#[test]
fn argent_dais_counts_any_players_attack() {
    cr!("508.3a", "603.2");
    ruling!(
        "Argent Dais",
        "Argent Dais's second ability triggers whenever two or more creatures attack, not just when you attack with two or more creatures."
    );
    supported("Argent Dais");
    let mut t = TestGame::new(2);
    let dais = t.battlefield(P1, "Argent Dais");
    let before = t.counters(dais, "oil");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(
        t.counters(dais, "oil"),
        before + 1,
        "an opponent's attack with two"
    );
    // One attacker: no trigger.
    let mut t = TestGame::new(2);
    let dais = t.battlefield(P1, "Argent Dais");
    let before = t.counters(dais, "oil");
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(dais, "oil"), before);
}

#[test]
fn lightmine_field_counts_attackers_as_it_resolves() {
    cr!("508.3a", "508.4", "506.4", "608.2h");
    ruling!(
        "Lightmine Field",
        "How much damage each of those creatures is dealt is based how many creatures are attacking at the time the ability resolves."
    );
    ruling!(
        "Lightmine Field",
        "that creature is dealt damage by Lightmine Field but isn’t counted when determining the amount of damage."
    );
    supported("Lightmine Field");
    // Three 4/4s attack; before the trigger resolves, one is removed from combat and a
    // Hill Giant enters attacking: still three attacking creatures, so each of the three
    // declared attackers (the removed one included) is dealt 3, the Giant none.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Lightmine Field");
    let a: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Siege Mastodon"))
        .collect();
    attack_with(
        &mut t,
        &a.iter()
            .map(|x| (*x, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    remove_from_combat(&mut t.g, a[0]);
    let giant = enter_attacking(&mut t, P0, "Hill Giant", Entity::Player(P1));
    t.resolve_all();
    assert!(!attacking(&t, a[0]));
    for x in &a {
        assert_eq!(damage_on(&t, *x), 3);
    }
    assert_eq!(damage_on(&t, giant), 0);
    // With nothing changed, two attackers are dealt 2 each.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Lightmine Field");
    let a: Vec<ObjectId> = (0..2)
        .map(|_| t.battlefield(P0, "Siege Mastodon"))
        .collect();
    attack_with(
        &mut t,
        &a.iter()
            .map(|x| (*x, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    t.resolve_all();
    assert_eq!(damage_on(&t, a[0]), 2);
}

#[test]
fn path_of_mettle_needs_two_attackers_each_with_one_of_the_abilities() {
    cr!("508.3c", "701.27a");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "any two attacking creatures you control need to each have any one of the four listed abilities. They don't need to share one of those abilities."
    );
    supported("Path of Mettle // Metzali, Tower of Triumph");
    let cases: [(&[&str], bool); 4] = [
        (&["Sun Sentinel", "Fanatical Firebrand"], true),
        (&["Sun Sentinel", "Sun Sentinel"], true),
        (&["Sun Sentinel", "Grizzly Bears"], false),
        (&["Sun Sentinel"], false),
    ];
    for (names, transforms) in cases {
        let mut t = TestGame::new(2);
        let path = t.battlefield(P0, "Path of Mettle // Metzali, Tower of Triumph");
        let atk: Vec<(ObjectId, Entity)> = names
            .iter()
            .map(|n| (t.battlefield(P0, n), Entity::Player(P1)))
            .collect();
        attack_with(&mut t, &atk);
        t.resolve_all();
        let name = t.obj_now(path).chars.name.to_string();
        assert_eq!(
            name == "Metzali, Tower of Triumph",
            transforms,
            "{names:?}: {name}"
        );
    }
}

/// Three players; P1 is the monarch and controls Emberwilde Captain, P0 attacks.
fn captain_game() -> TestGame {
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Emberwilde Captain");
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.settle();
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t
}

#[test]
fn emberwilde_captain_triggers_once_per_attacking_player_attacking_you() {
    cr!("508.3e", "603.2", "603.4");
    ruling!(
        "Emberwilde Captain",
        "Emberwilde Captain's last ability triggers once for each player who attacks you, no matter how many creatures they attack with beyond the first."
    );
    ruling!(
        "Emberwilde Captain",
        "Emberwilde Captain's last ability won't trigger if an opponent attacks only planeswalkers you control and not you."
    );
    supported("Emberwilde Captain");
    // Two attackers at P1: one trigger, 3 damage (P0 has three cards in hand).
    let mut t = captain_game();
    let hand = t.hand_size(P0) as i32;
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 - hand);
    // Only P1's planeswalker attacked: no trigger.
    let mut t = captain_game();
    let pw = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, obj(pw))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn emberwilde_captain_resolves_after_the_monarch_changes() {
    cr!("603.2", "725.3");
    ruling!(
        "Emberwilde Captain",
        "If another player becomes the monarch after Emberwilde Captain's last ability triggers but before it resolves, the ability will resolve as normal."
    );
    // The attacking player becomes the monarch with the trigger on the stack.
    let mut t = captain_game();
    let hand = t.hand_size(P0) as i32;
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert_eq!(
        t.life(P0),
        20 - hand,
        "the attacking player was still dealt damage"
    );
}

#[test]
fn kazuul_always_offers_the_attacking_player_the_payment() {
    cr!("118.12", "508.3a", "603.4");
    ruling!(
        "Kazuul, Tyrant of the Cliffs",
        "As the ability resolves, you can't put the Ogre token onto the battlefield without explicitly giving the attacking player the option to pay {3}"
    );
    supported("Kazuul, Tyrant of the Cliffs");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Kazuul, Tyrant of the Cliffs");
        let a = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Wastes", 3);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        let from = t.asked().len();
        t.answer_yes(P0, pays);
        t.resolve_all();
        let asked_p0 = t.asked()[from..]
            .iter()
            .filter(|(p, d)| {
                *p == P0 && !matches!(d, mtg_engine::decision::Decision::Priority { .. })
            })
            .count();
        assert!(asked_p0 >= 1, "the attacking player was asked");
        let ogres = crate::r_s01_common::with_subtype(&t, P1, "Ogre").len();
        assert_eq!(ogres, if pays { 1 } else { 2 }, "pays: {pays}");
    }
}

#[test]
fn kazuul_trigger_resolves_before_the_attackers_own_trigger() {
    cr!("603.3b", "101.4");
    ruling!(
        "Kazuul, Tyrant of the Cliffs",
        "Although Kazuul's ability will resolve first, and potentially create an Ogre token, any target of the other ability will already have been chosen"
    );
    supported("Goblin Heelcutter");
    // P0's Goblin Heelcutter attacks P1 ("target creature can't block this turn", aimed at
    // P1's Bears). P0 is the active player, so Kazuul's trigger (P1's) is put on the stack
    // last and resolves first; the Ogre it creates can still block.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Kazuul, Tyrant of the Cliffs");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let heel = t.battlefield(P0, "Goblin Heelcutter");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(heel, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    let top = crate::r_s04_common::top_of_stack(&t);
    assert_eq!(t.obj_now(top).controller, P1, "Kazuul's trigger is on top");
    t.answer_yes(P0, false);
    t.resolve();
    let ogre = crate::r_s01_common::with_subtype(&t, P1, "Ogre")
        .into_iter()
        .find(|o| t.obj_now(*o).is_token())
        .expect("Ogre token");
    t.resolve_all();
    t.g.recompute();
    assert!(!t.g.can_block_at_all(bears), "the Heelcutter's target");
    assert!(
        t.g.can_block_at_all(ogre),
        "the Ogre wasn't a possible target"
    );
}

#[test]
fn mangara_uses_last_known_attack_target_of_creatures_that_left() {
    cr!("603.4", "608.2h", "506.4");
    ruling!(
        "Mangara, the Diplomat",
        "If some of the attacking creatures leave the battlefield while Mangara's triggered ability is on the stack, use the player or planeswalker they were attacking before they left"
    );
    supported("Mangara, the Diplomat");
    supported("Tomik, Wielder of Law");
    // P0 attacks P1 (Mangara, or Tomik with the same condition) with two creatures; with
    // the trigger on the stack one of them dies (P1 still draws) or is removed from combat
    // (P1 doesn't draw), or is removed from combat and then dies (it wasn't attacking as
    // it last existed: P1 doesn't draw).
    for card in ["Mangara, the Diplomat", "Tomik, Wielder of Law"] {
        for (dies, removed) in [(true, false), (false, true), (true, true)] {
            let mut t = TestGame::new(2);
            t.battlefield(P1, card);
            let a = t.battlefield(P0, "Grizzly Bears");
            let b = t.battlefield(P0, "Hill Giant");
            attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
            assert_eq!(t.stack_len(), 1);
            if removed {
                remove_from_combat(&mut t.g, a);
            }
            if dies {
                crate::r_s02_common::destroy(&mut t, a);
            }
            let hand = t.hand_size(P1);
            t.resolve_all();
            let draws = dies && !removed;
            assert_eq!(
                t.hand_size(P1),
                hand + draws as usize,
                "{card} dies: {dies}, removed: {removed}"
            );
        }
    }
    // A planeswalker attacked counts too, even after the attacker died.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mangara, the Diplomat");
    let pw = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, obj(pw)), (b, Entity::Player(P1))]);
    crate::r_s02_common::destroy(&mut t, a);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    // Two attackers, only one at P1 (the other at P2): no trigger resolution draw.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Mangara, the Diplomat");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P1))]);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn lost_in_the_woods_each_trigger_is_tied_to_its_attacker() {
    cr!("508.3a", "603.2", "506.4");
    ruling!(
        "Lost in the Woods",
        "Because Lost in the Woods will trigger once for each creature that attacks you or a planeswalker you control"
    );
    supported("Lost in the Woods");
    // Two attackers, two triggers; P1's library has a Forest on top and an Island below.
    // The trigger resolving first reveals the Forest and removes its own attacker.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Lost in the Woods");
    crate::r_s01_common::stack_library(&mut t, P1, &["Forest", "Island"]);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    let top = crate::r_s04_common::top_of_stack(&t);
    let its = t
        .obj_now(top)
        .stack
        .as_ref()
        .unwrap()
        .event
        .as_ref()
        .unwrap()
        .object
        .unwrap();
    let other = if its == a { b } else { a };
    t.resolve();
    assert!(
        !attacking(&t, its),
        "the trigger's own creature left combat"
    );
    assert!(attacking(&t, other));
    t.resolve_all();
    assert!(
        attacking(&t, other),
        "the Island was revealed for the other one"
    );
}

/// P0 controls O-Kagachi. During P1's turn `p1_turn` runs in P1's declare attackers step
/// (P1 has a Grizzly Bears to attack with); then, in P0's next turn, O-Kagachi attacks P1
/// unblocked, targeting P1's Hill Giant if it triggers. Returns whether the Giant was
/// exiled.
fn okagachi_after(players: usize, p1_turn: fn(&mut TestGame, ObjectId)) -> bool {
    let mut t = TestGame::new(players);
    let ok = t.battlefield(P0, "O-Kagachi, Vengeful Kami");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, mtg_engine::turn::Step::BeginningOfCombat);
    p1_turn(&mut t, bears);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(giant)]);
    t.attack(&[(ok, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.life(P1) <= 14, "O-Kagachi dealt combat damage to P1");
    !t.on_battlefield(giant)
}

#[test]
fn okagachi_player_attacked_you_only_if_a_declared_attacker_attacks_you() {
    cr!("508.6", "508.4", "508.7", "603.4");
    ruling!(
        "O-Kagachi, Vengeful Kami",
        "A player attacked you if that player declared a creature as an attacking creature and that creature is attacking you."
    );
    supported("O-Kagachi, Vengeful Kami");
    // Declared attacking P0: triggers.
    assert!(okagachi_after(2, |t, bears| {
        attack_with(t, &[(bears, Entity::Player(P0))]);
    }));
    // Attacked only P0's planeswalker: no.
    assert!(!okagachi_after(2, |t, bears| {
        let pw = t.battlefield(P0, "Jace Beleren");
        attack_with(t, &[(bears, obj(pw))]);
    }));
    // Attacked P2; a creature put onto the battlefield attacking P0: no.
    assert!(!okagachi_after(3, |t, bears| {
        attack_with(t, &[(bears, Entity::Player(P2))]);
        let new = enter_attacking(t, P1, "Savannah Lions", Entity::Player(P0));
        assert!(attacking(t, new));
    }));
    // Attacked P2, and P0 reselected P0 as the player the Bears attack: no.
    assert!(!okagachi_after(3, |t, bears| {
        attack_with(t, &[(bears, Entity::Player(P2))]);
        crate::r_s01_common::give_mana_for(t, P0, "Portal Mage");
        let mage = t.hand(P0, "Portal Mage");
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_choose(P0, &[Entity::Player(P0)]);
        t.cast(P0, mage).go();
        t.resolve();
        t.resolve();
        assert_eq!(
            t.g.combat.as_ref().unwrap().attack_target(bears),
            Some(Entity::Player(P0))
        );
    }));
}

#[test]
fn okagachi_blocker_dies_before_the_target_is_chosen() {
    cr!("510.2", "704.5g", "603.3d", "702.19b");
    ruling!(
        "O-Kagachi, Vengeful Kami",
        "that blocking creature is destroyed before a target is selected for O-Kagachi's last ability. It can't be exiled."
    );
    // P1 attacked P0 last turn; now O-Kagachi (6/6 flying, trample) is blocked by P1's
    // Wind Drake: 2 to the Drake, 4 to P1. The Drake is already dead when the target is
    // chosen.
    let mut t = TestGame::new(2);
    let ok = t.battlefield(P0, "O-Kagachi, Vengeful Kami");
    let bears = t.battlefield(P1, "Wind Drake");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.advance_to(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(lions, Entity::Player(P0))]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    let from = t.asked().len();
    t.attack(&[(ok, Entity::Player(P1))], &[(bears, ok)]);
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(bears));
    let candidates = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!candidates.is_empty(), "the trigger asked for a target");
    assert!(candidates.iter().all(|c| !c.contains(&obj(bears))));
}

#[test]
fn reselected_defending_player_makes_targets_of_defending_players_permanents_illegal() {
    cr!("508.7", "608.2b");
    ruling!(
        "Portal Mage",
        "If an ability targets something controlled by the “defending player” of an attacking creature and the defending player for that creature changes before that ability resolves, the ability won’t resolve"
    );
    ruling!(
        "Windshaper Planetar",
        "If an ability targets something controlled by the “defending player” of an attacking creature and the defending player for that creature changes before that ability resolves, the ability won't resolve"
    );
    supported("Master of Diversion");
    // Master of Diversion ("whenever this creature attacks, tap target creature defending
    // player controls") attacks P1, aimed at P1's Bears; P1 reselects it to attack P2.
    for planetar in [false, true] {
        let mut t = TestGame::new(3);
        let master = t.battlefield(P0, "Master of Diversion");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &[(master, Entity::Player(P1))]);
        let name = if planetar {
            "Windshaper Planetar"
        } else {
            "Portal Mage"
        };
        crate::r_s01_common::give_mana_for(&mut t, P1, name);
        let card = t.hand(P1, name);
        if !planetar {
            t.answer_targets(P1, &[obj(master)]);
        }
        t.answer_choose(P1, &[Entity::Player(P2)]);
        t.cast(P1, card).go();
        t.resolve();
        t.resolve();
        assert_eq!(
            t.g.combat.as_ref().unwrap().attack_target(master),
            Some(Entity::Player(P2)),
            "{name}"
        );
        t.resolve_all();
        assert!(!crate::r_s08_common::is_tapped(&t, bears), "{name}");
    }
}

#[test]
fn sarkhan_planeswalker_creatures_need_to_have_been_controlled_since_the_turn_began() {
    cr!("302.6", "508.1a");
    ruling!(
        "Sarkhan the Masterless",
        "A planeswalker that becomes a creature due to Sarkhan’s first loyalty ability can’t attack unless you’ve controlled it continuously since your turn began."
    );
    supported("Sarkhan the Masterless");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let sarkhan = t.battlefield(P0, "Sarkhan the Masterless");
    let jace = t.battlefield_sick(P0, "Jace Beleren");
    t.activate(P0, sarkhan, 0, &[]).unwrap();
    t.resolve_all();
    assert!(crate::r_s04_common::is_creature(&t, jace));
    assert!(crate::r_s04_common::is_creature(&t, sarkhan));
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, sarkhan));
    assert!(
        !crate::r_s02_common::can_attack(&mut t, jace),
        "came under P0's control this turn"
    );
}

#[test]
fn mirkwood_trapper_attacking_only_your_planeswalker_isnt_attacking_you() {
    cr!("508.3d", "603.4", "506.2");
    ruling!(
        "Mirkwood Trapper",
        "If a player is attacking a planeswalker you control or a battle you're protecting but not attacking you, Mirkwood Trapper's last ability will still trigger."
    );
    supported("Mirkwood Trapper");
    // P0 attacks only P1's planeswalker: the last ability (not the first) triggers, and
    // P0's attacking Bears get +2/+0.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mirkwood Trapper");
    let pw = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    attack_with(&mut t, &[(bears, obj(pw))]);
    assert_eq!(t.stack_len(), 1, "only the last ability");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // Attacking P1: the first ability (-2/-0 to target attacking creature) instead.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mirkwood Trapper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[obj(bears)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (0, 2));
}

#[test]
fn imprison_abilities_are_triggered_abilities() {
    cr!("603.1", "603.2", "508.3a");
    ruling!(
        "Imprison",
        "Both abilities are triggered abilities. The first one triggers on the activating of a creature ability, and the second triggers on declaration of attacking or blocking."
    );
    supported("Imprison");
    // P1's Imprison on P0's Prodigal Pyromancer: activating its {T} ability triggers it
    // (the activation is still on the stack under the trigger); so does attacking.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let aura = t.battlefield(P1, "Imprison");
    assert!(t.g.attach(aura, obj(pyro)));
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s06_common::activate_containing(&mut t, P0, pyro, "damage").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the activation and Imprison's trigger");
    let top = crate::r_s04_common::top_of_stack(&t);
    assert_eq!(t.obj_now(top).controller, P1);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(!t.on_battlefield(aura), "not paid: Imprison is destroyed");
    assert_eq!(t.life(P1), 19, "the ability wasn't countered");
    // Declared as an attacker: the second trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P1, "Imprison");
    assert!(t.g.attach(aura, obj(bears)));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(!t.on_battlefield(aura));
}
