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
    assert_eq!(t.life(P0), 17, "one trigger per creature, the planeswalker's included");
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
        !t.obj_now(angel).has_keyword(mtg_engine::keywords::KeywordKind::Flying),
        "Gravity Well removed its flying"
    );
    assert_eq!(damage_on(&t, angel), 0, "no trigger for the creature that attacked flying");
    assert_eq!(damage_on(&t, bears), 1, "the creature without flying was dealt 1");
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
    let flying =
        |t: &TestGame, id| t.obj_now(id).has_keyword(mtg_engine::keywords::KeywordKind::Flying);
    assert!(flying(&t, cricket), "the Cricket's own attack trigger resolved last");
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
    assert_eq!(t.counters(dais, "oil"), before + 1, "an opponent's attack with two");
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
    let a: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Siege Mastodon")).collect();
    attack_with(
        &mut t,
        &a.iter().map(|x| (*x, Entity::Player(P1))).collect::<Vec<_>>(),
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
    let a: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Siege Mastodon")).collect();
    attack_with(
        &mut t,
        &a.iter().map(|x| (*x, Entity::Player(P1))).collect::<Vec<_>>(),
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
    assert_eq!(t.life(P0), 20 - hand, "the attacking player was still dealt damage");
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
            .filter(|(p, d)| *p == P0 && !matches!(d, mtg_engine::decision::Decision::Priority { .. }))
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
    assert!(t.g.can_block_at_all(ogre), "the Ogre wasn't a possible target");
}
