//! Rulings batch P018 — combat and ramp: permission to play a card from a graveyard
//! (CR 601.2, 305.1, 305.2), putting lands onto the battlefield isn't playing them
//! (CR 305.4), optional "you may put it onto the battlefield" (CR 609.3), mana emptying
//! between steps (CR 500.4), "becomes blocked" by an effect (CR 509.1h, 702.19e,
//! 702.45a), illegal targets (CR 608.2b), the same object for one or several "target"
//! words (CR 115.3), and effects on all players.

use crate::r_p018_common::*;
use crate::r_p212_common::back_face;
use crate::r_s01_common::{attack_with, stack_library, supported, tokens};
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s04_common::{add_mana, on_stack};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::{tap_for_mana, to_beginning_of_combat};
use crate::r_s25_common::targets_of;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    plenty_of_mana(t, p);
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, card).go();
    empty_pool(t, p);
    t.resolve_all();
}

// ---------------------------------------------------------------------------
// Crumbling Sanctuary, Dueling Grounds: "The ability affects all players."
// ---------------------------------------------------------------------------

#[test]
fn crumbling_sanctuary_and_dueling_grounds_affect_all_players() {
    cr!("614.1a", "508.1c");
    ruling!("Crumbling Sanctuary", "The ability affects all players.");
    ruling!("Dueling Grounds", "The ability affects all players.");
    supported("Crumbling Sanctuary");
    supported("Dueling Grounds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crumbling Sanctuary");
    let lib0 = t.library_size(P0);
    let lib1 = t.library_size(P1);
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P0)]);
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.library_size(P0), lib0 - 2);
    assert_eq!(t.library_size(P1), lib1 - 2);
    // Dueling Grounds limits each player's attacks.
    for p in [P0, P1] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Dueling Grounds");
        let a = t.battlefield(p, "Grizzly Bears");
        let b = t.battlefield(p, "Grizzly Bears");
        let other = if p == P0 { P1 } else { P0 };
        to_beginning_of_combat(&mut t, p);
        assert!(legal_attack(&mut t, &[(a, Entity::Player(other))]));
        assert!(!legal_attack(
            &mut t,
            &[(a, Entity::Player(other)), (b, Entity::Player(other))]
        ));
    }
}

// ---------------------------------------------------------------------------
// Jorn, God of Winter // Kaldring, the Rimestaff
// ---------------------------------------------------------------------------

const JORN: &str = "Jorn, God of Winter // Kaldring, the Rimestaff";

/// P0's Kaldring targets the snow permanent card `card` in P0's graveyard, resolving.
fn kaldring(t: &mut TestGame, card: ObjectId) -> ObjectId {
    let staff = back_face(t, P0, JORN);
    t.activate(P0, staff, 0, &[Entity::Object(card)]).unwrap();
    t.resolve();
    staff
}

#[test]
fn kaldring_lets_you_play_the_card_but_it_stays_in_the_graveyard() {
    cr!("601.2", "601.3", "305.1");
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "Even though you may play the card, it isn't in your hand, so you can't, for example, discard it or foretell it."
    );
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "You don't play the card as Kaldring's activated ability is resolving; rather, that ability gives you permission to play that card that turn."
    );
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "You'll still pay all costs to cast a spell this way, including any additional costs. You may also pay alternative costs if any are available."
    );
    supported(JORN);
    supported("Icehide Troll");
    let mut t = TestGame::new(2);
    let troll = t.graveyard(P0, "Icehide Troll");
    let hand = t.hand_size(P0);
    kaldring(&mut t, troll);
    // Nothing was played: it's still in the graveyard, not in hand.
    assert_eq!(t.zone(troll), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand);
    // Without mana, it can't be cast; with {2}{G}, it's cast paying it, and enters tapped.
    assert!(!can_cast(&mut t, P0, troll, CastMethod::Normal));
    add_mana(&mut t, P0, ManaType::G, 3);
    assert!(can_cast(&mut t, P0, troll, CastMethod::Normal));
    t.cast(P0, troll).go();
    assert_eq!(t.g.players[P0.idx()].mana_pool.total(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(troll));
    assert!(t.obj_now(troll).tapped);
}

#[test]
fn kaldrings_card_left_unplayed_stays_in_the_graveyard() {
    cr!("611.2a");
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "If you don't end up playing the card, it remains in your graveyard. You can't play it on future turns unless you target it again with Kaldring's ability (or something else lets you play it)."
    );
    let mut t = TestGame::new(2);
    let troll = t.graveyard(P0, "Icehide Troll");
    kaldring(&mut t, troll);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(troll), Zone::Graveyard(P0));
    add_mana(&mut t, P0, ManaType::G, 3);
    assert!(!can_cast(&mut t, P0, troll, CastMethod::Normal));
}

#[test]
fn kaldring_follows_normal_timing_and_land_plays() {
    cr!("305.2", "305.3", "307.1", "601.3");
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "You must follow the normal timing permissions and restrictions for the card when you play it. To play a land this way, you must have a land play available. Notably, if you activate Kaldring's ability during an opponent's turn, you won't be able to cast a spell that way unless it has flash and you won't be able to play a land that way at all."
    );
    supported("Snow-Covered Forest");
    // On an opponent's turn: neither the Troll nor the land.
    for name in ["Icehide Troll", "Snow-Covered Forest"] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let card = t.graveyard(P0, name);
        t.g.turn.priority = Some(P0);
        kaldring(&mut t, card);
        add_mana(&mut t, P0, ManaType::G, 3);
        t.g.turn.priority = Some(P0);
        let ok = if name == "Icehide Troll" {
            can_cast(&mut t, P0, card, CastMethod::Normal)
        } else {
            can_play_land(&mut t, P0, card)
        };
        assert!(!ok, "{name} on an opponent's turn");
    }
    // On your turn, with your land play used: not the land.
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Snow-Covered Forest");
    kaldring(&mut t, forest);
    assert!(can_play_land(&mut t, P0, forest));
    let other = t.hand(P0, "Forest");
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(&mut t, P0, forest));
    // A fresh one, with the land play available: it's played, tapped.
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Snow-Covered Forest");
    kaldring(&mut t, forest);
    t.play_land(P0, forest).unwrap();
    t.settle();
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
}

#[test]
fn jorns_attack_trigger_untaps_jorn_itself() {
    cr!("603.2", "508.1f");
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "Jorn's triggered ability will untap Jorn itself."
    );
    let mut t = TestGame::new(2);
    let jorn = t.battlefield(P0, JORN);
    attack_with(&mut t, &[(jorn, Entity::Player(P1))]);
    assert!(t.obj_now(jorn).tapped);
    t.resolve_all();
    assert!(!t.obj_now(jorn).tapped);
    assert!(t.g.is_attacking(jorn));
}

// ---------------------------------------------------------------------------
// Putting lands onto the battlefield.
// ---------------------------------------------------------------------------

#[test]
fn rulik_mons_may_leave_the_land_on_top_and_make_a_goblin() {
    cr!("608.2c", "305.4");
    ruling!(
        "Rulik Mons, Warren Chief",
        "If it is a land, you may choose not to put it onto the battlefield this way. In that case, you will create a 1/1 red Goblin creature token."
    );
    ruling!(
        "Rulik Mons, Warren Chief",
        "Putting a land onto the battlefield with Rulik Mons, Warren Chief's second ability doesn't count as playing a land for the turn."
    );
    supported("Rulik Mons, Warren Chief");
    for put in [false, true] {
        let mut t = TestGame::new(2);
        let rulik = t.battlefield(P0, "Rulik Mons, Warren Chief");
        let land = stack_library(&mut t, P0, &["Mountain"])[0];
        t.answer_yes(P0, put);
        attack_with(&mut t, &[(rulik, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(land), put);
        assert_eq!(tokens(&t, P0).len(), usize::from(!put));
        if put {
            assert!(t.obj_now(land).tapped);
            assert_eq!(t.g.players[P0.idx()].lands_played_this_turn, 0);
        } else {
            assert_eq!(t.zone(land), Zone::Library(P0));
        }
    }
}

#[test]
fn michelangelo_puts_a_land_even_after_your_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Michelangelo, Improviser",
        "Putting a land card onto the battlefield with Michelangelo's last ability doesn't count as playing a land. You can put a land card onto the battlefield this way even if you've already played a land for the turn."
    );
    supported("Michelangelo, Improviser");
    let mut t = TestGame::new(2);
    let mikey = t.battlefield(P0, "Michelangelo, Improviser");
    let played = t.hand(P0, "Forest");
    t.play_land(P0, played).unwrap();
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_yes(P0, true);
    t.attack(&[(mikey, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.players[P0.idx()].lands_played_this_turn, 1);
}

// ---------------------------------------------------------------------------
// Sword of Feast and Famine: "Whenever equipped creature deals combat damage to a
// player, that player discards a card and you untap all lands you control."
// ---------------------------------------------------------------------------

/// P0's Grizzly Bears with the Sword attack P1 unblocked; the game stops in the combat
/// damage step with the Sword's trigger on the stack.
fn sword_hit(t: &mut TestGame) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Sword of Feast and Famine");
    assert!(t.g.attach(sword, Entity::Object(bears)));
    attack_with(t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(on_stack(t, "untap all lands"), 1);
}

#[test]
fn sword_of_feast_and_famine_mana_tapped_in_response_empties_after_the_step() {
    cr!("500.5", "106.4");
    ruling!(
        "Sword of Feast and Famine",
        "You can tap your lands for mana while the triggered ability is on the stack. However, you'll lose any mana you produce this way if you don't spend it before the end of the current combat damage step."
    );
    supported("Sword of Feast and Famine");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 2);
    t.hand(P1, "Grizzly Bears");
    sword_hit(&mut t);
    for l in &lands {
        assert!(tap_for_mana(&mut t, P0, *l, "Add"));
    }
    t.resolve();
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
    assert_eq!(t.g.players[P0.idx()].mana_pool.total(), 2);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.g.players[P0.idx()].mana_pool.total(), 0);
}

#[test]
fn sword_of_feast_and_famine_untaps_lands_even_if_no_card_is_discarded() {
    cr!("609.3");
    ruling!(
        "Sword of Feast and Famine",
        "You'll untap all lands you control even if the player can't discard a card."
    );
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 2);
    for l in &lands {
        t.g.tap(*l);
    }
    assert_eq!(t.hand_size(P1), 0);
    sword_hit(&mut t);
    t.resolve();
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
}

#[test]
fn fishing_gear_may_leave_the_permanent_card_in_exile_for_a_fish() {
    cr!("608.2c");
    ruling!(
        "Fishing Gear",
        "You don't have to put the card on the battlefield even if it's a permanent card. You may choose to leave it in exile and create a 1/1 blue Fish token."
    );
    supported("Fishing Gear");
    for put in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let gear = t.battlefield(P0, "Fishing Gear");
        assert!(t.g.attach(gear, Entity::Object(bears)));
        let top = stack_library(&mut t, P1, &["Hill Giant"])[0];
        t.answer_yes(P0, put);
        t.attack(&[(bears, Entity::Player(P1))], &[]);
        if put {
            assert!(t.on_battlefield(top));
            assert_eq!(t.obj_now(top).controller, P0);
            assert!(tokens(&t, P0).is_empty());
        } else {
            assert_eq!(t.zone(top), Zone::Exile);
            let fish = tokens(&t, P0);
            assert_eq!(fish.len(), 1);
            assert!(t.obj_now(fish[0]).chars.has_subtype("Fish"));
        }
    }
}

// ---------------------------------------------------------------------------
// Curtain of Light: "Cast this spell only during combat after blockers are declared.
// Target unblocked attacking creature becomes blocked. Draw a card."
// ---------------------------------------------------------------------------

/// P0 attacks P1 with `attacker` unblocked; the game stops in the declare blockers step.
fn unblocked(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(!t.g.combat.as_ref().is_some_and(|c| c.is_blocked(attacker)));
}

fn curtain(t: &mut TestGame, attacker: ObjectId) {
    add_mana(t, P1, ManaType::W, 2);
    let c = t.hand(P1, "Curtain of Light");
    t.cast(P1, c).target(attacker).go();
    t.resolve_all();
}

#[test]
fn curtain_of_light_has_targets_only_during_combat() {
    cr!("506.4", "509.1h", "115.1");
    ruling!(
        "Curtain of Light",
        "An attacking unblocked creature exists from the time the defending player chooses not to block that creature until the combat phase ends. Curtain of Light doesn’t have any legal targets outside of the combat phase."
    );
    supported("Curtain of Light");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P1, "Curtain of Light");
    add_mana(&mut t, P1, ManaType::W, 2);
    t.g.turn.priority = Some(P1);
    assert!(!can_cast(&mut t, P1, c, CastMethod::Normal));
    unblocked(&mut t, bears);
    // Still an unblocked attacker in the end of combat step.
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.g.is_attacking(bears) && !t.g.combat.as_ref().is_some_and(|c| c.is_blocked(bears)));
    add_mana(&mut t, P1, ManaType::W, 2);
    t.g.turn.priority = Some(P1);
    assert!(can_cast(&mut t, P1, c, CastMethod::Normal));
    t.cast(P1, c).target(bears).go();
    t.resolve_all();
    assert!(t.g.combat.as_ref().is_some_and(|c| c.is_blocked(bears)));
    // After combat, no target.
    t.advance_to(P0, Step::PostcombatMain);
    let c2 = t.hand(P1, "Curtain of Light");
    add_mana(&mut t, P1, ManaType::W, 2);
    t.g.turn.priority = Some(P1);
    assert!(!can_cast(&mut t, P1, c2, CastMethod::Normal));
}

#[test]
fn curtain_of_light_triggers_bushido_and_trample_hits_the_player() {
    cr!("509.1h", "509.3c", "702.45a", "702.19d");
    ruling!(
        "Curtain of Light",
        "Curtain of Light does trigger bushido abilities."
    );
    ruling!(
        "Curtain of Light",
        "Curtain of Light isn’t particularly effective against creatures with trample. Since the creature with trample has no blocker to assign damage to, all the damage will be assigned to the defending player."
    );
    supported("Devoted Retainer");
    let mut t = TestGame::new(2);
    let retainer = t.battlefield(P0, "Devoted Retainer");
    unblocked(&mut t, retainer);
    curtain(&mut t, retainer);
    assert!(t.g.combat.as_ref().is_some_and(|c| c.is_blocked(retainer)));
    assert_eq!(t.pt(retainer), (2, 2), "bushido 1 triggered");
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Trample: all of it to the player.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Colossal Dreadmaw");
    unblocked(&mut t, wurm);
    curtain(&mut t, wurm);
    assert!(t.g.combat.as_ref().is_some_and(|c| c.is_blocked(wurm)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

// ---------------------------------------------------------------------------
// Combat tricks.
// ---------------------------------------------------------------------------

#[test]
fn phalanx_tactics_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Phalanx Tactics",
        "If the target creature is an illegal target by the time Phalanx Tactics tries to resolve, the spell won’t resolve. No creatures get +1/+1."
    );
    supported("Phalanx Tactics");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    plenty_of_mana(&mut t, P0);
    let tactics = t.hand(P0, "Phalanx Tactics");
    t.cast(P0, tactics).target(a).go();
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(a).go();
    empty_pool(&mut t, P0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.pt(b), (2, 2));
}

#[test]
fn nahiris_stoneblades_cant_target_one_creature_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Nahiri's Stoneblades",
        "You can’t target the same creature twice to have it get +4/+0."
    );
    supported("Nahiri's Stoneblades");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    plenty_of_mana(&mut t, P0);
    let blades = t.hand(P0, "Nahiri's Stoneblades");
    // Naming the same creature twice isn't a legal choice: the spell doesn't target it
    // twice, and it can't get +4/+0.
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(a)]);
    let spell = t.cast(P0, blades).go();
    let targeted = targets_of(&t, spell);
    assert!(targeted.iter().filter(|e| **e == Entity::Object(a)).count() <= 1);
    empty_pool(&mut t, P0);
    t.resolve_all();
    assert!(t.pt(a).0 <= 4);
    let blades = t.hand(P0, "Nahiri's Stoneblades");
    plenty_of_mana(&mut t, P0);
    t.cast(P0, blades)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.pt(b), (4, 2));
}

#[test]
fn martial_glory_may_target_the_same_creature_twice() {
    cr!("115.3");
    ruling!(
        "Martial Glory",
        "You may choose the same creature for both targets since the card says \"target creature\" multiple times. You may also choose two different creatures."
    );
    supported("Martial Glory");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(
        &mut t,
        P0,
        "Martial Glory",
        &[Entity::Object(a), Entity::Object(a)],
    );
    assert_eq!(t.pt(a), (5, 5));
    cast_resolve(
        &mut t,
        P0,
        "Martial Glory",
        &[Entity::Object(a), Entity::Object(b)],
    );
    assert_eq!(t.pt(a), (8, 5));
    assert_eq!(t.pt(b), (2, 5));
    let _ = Answer::Default;
}
