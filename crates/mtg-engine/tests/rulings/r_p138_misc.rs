//! Rulings batch P138 — the rest: Tainted Aether's triggers, edicts and state-based
//! actions, friend or foe, multiplayer players leaving (CR 800.4), modes chosen more than
//! once (CR 700.2d), casting from outside the game, and cards that restate their text.

use crate::r_p138_common::*;
use crate::r_s01_common::{place, tokens};
use crate::r_s17_common::put_onto_battlefield;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Tainted Aether
// ---------------------------------------------------------------------------

#[test]
fn tainted_aether_each_copy_triggers() {
    cr!("603.2", "603.6a");
    ruling!(
        "Tainted Aether",
        "If more than one of these is on the battlefield, the controller sacrifices a creature for each one of these."
    );
    supported("Tainted Aether");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tainted Aether");
    t.battlefield(P0, "Tainted Aether");
    t.lands(P1, "Forest", 3);
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P1).len(), 2, "two of four sacrificed");
}

#[test]
fn tainted_aether_the_entering_creature_may_have_to_be_sacrificed() {
    cr!("701.21a", "603.6a");
    ruling!(
        "Tainted Aether",
        "If the controller has no other lands or creatures on the battlefield, they must sacrifice the creature that just entered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tainted Aether");
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn tainted_aether_its_controller_controls_the_trigger_and_the_creatures_chooses() {
    cr!("603.3a", "608.2d", "113.8");
    ruling!(
        "Tainted Aether",
        "Tainted Aether's controller is the one that controls its triggered ability, but the controller of the creature that entered chooses what creature they sacrifice during resolution."
    );
    ruling!(
        "Tainted Aether",
        "If the creature changes controllers before this ability resolves, its new controller has to sacrifice something."
    );
    // P1's creature enters; the trigger is P0's and P1 chooses.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tainted Aether");
    let forest = t.battlefield(P1, "Forest");
    t.enter(P1, "Grizzly Bears");
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(trig).controller, P0);
    t.answer_choose(P1, &[obj(forest)]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P1]);
    assert!(!t.on_battlefield(forest));
    // The creature changes controllers first: its new controller (P0) sacrifices.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tainted Aether");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Forest");
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    cast(&mut t, P0, "Ray of Command", &[&[obj(bears)]]);
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.answer_choose(P0, &[obj(mine)]);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}

// ---------------------------------------------------------------------------
// Edicts and state-based actions; cards that restate their text
// ---------------------------------------------------------------------------

#[test]
fn ferocidons_creatures_dealt_lethal_damage_die_before_the_choice() {
    cr!("704.3", "603.3", "704.5g");
    ruling!(
        "Silverclad Ferocidons",
        "If creatures an opponent controls are dealt lethal damage at the same time that Silverclad Ferocidons is dealt damage, those creatures will be destroyed before that player chooses a permanent to sacrifice."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverclad Ferocidons");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let from = t.asked().len();
    cast(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(!first_offer(&t, P1, from).contains(&bears));
    assert!(!t.on_battlefield(forest), "the Forest was the only choice");
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

#[test]
fn tyrants_choice_death_doesnt_target() {
    cr!("701.38a", "702.16b", "115.1");
    ruling!(
        "Tyrant's Choice",
        "If death gets more votes, each opponent chooses and sacrifices a creature they control. None of these creatures are targeted. An opponent could sacrifice a creature with protection from black, for example."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    cast(&mut t, P0, "Tyrant's Choice", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(knight));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn structural_collapse_targets_only_the_player() {
    cr!("115.1", "702.11b");
    ruling!(
        "Structural Collapse",
        "Structural Collapse targets only the player. The player can sacrifice an artifact or land with hexproof, for example."
    );
    let mut t = TestGame::new(2);
    let field = t.battlefield(P1, "Lotus Field");
    let ring = t.battlefield(P1, "Sol Ring");
    cast(&mut t, P0, "Structural Collapse", &[&[pl(P1)]]);
    t.resolve_all();
    assert!(!t.on_battlefield(field) && !t.on_battlefield(ring));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn players_without_the_permanents_do_nothing() {
    cr!("701.21a", "609.3");
    ruling!(
        "Stoneshaker Shaman",
        "If the player doesn’t have an untapped land as Stoneshaker Shaman’s ability resolves, that player does nothing."
    );
    ruling!(
        "Destructive Urge",
        "The ability has no effect if the player has no lands."
    );
    supported("Stoneshaker Shaman");
    supported("Destructive Urge");
    // Stoneshaker Shaman: P0's lands are all tapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stoneshaker Shaman");
    let lands = t.lands(P0, "Mountain", 2);
    for l in &lands {
        t.g.tap(*l);
    }
    to_end_step(&mut t);
    t.resolve_all();
    assert!(lands.iter().all(|l| t.on_battlefield(*l)));
    // Destructive Urge: P1 has no lands.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let urge = t.battlefield(P0, "Destructive Urge");
    assert!(t.g.attach(urge, obj(bears)));
    let theirs = t.battlefield(P1, "Sol Ring");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(theirs));
}

#[test]
fn the_eternal_wanderer_has_no_planeswalker_type() {
    cr!("205.3j", "306.3");
    ruling!(
        "The Eternal Wanderer",
        "The Eternal Wanderer does not have a planeswalker type."
    );
    supported("The Eternal Wanderer");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "The Eternal Wanderer");
    assert!(t.obj(w).is(CardType::Planeswalker));
    assert!(t.obj(w).chars.subtypes.is_empty());
}

#[test]
fn barrensteppe_siege_without_a_choice_has_neither_ability() {
    cr!("614.12c");
    ruling!(
        "Barrensteppe Siege",
        "If you somehow control Barrensteppe Siege and no choice was made for it (perhaps because another permanent on the battlefield became a copy of it), it has neither of the two abilities."
    );
    supported("Barrensteppe Siege");
    let mut t = TestGame::new(2);
    // Put onto the battlefield without entering (no choice made).
    t.battlefield(P0, "Barrensteppe Siege");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dying = t.battlefield(P0, "Hill Giant");
    kill(&mut t, dying);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert!(t.on_battlefield(theirs));
}

#[test]
fn slave_of_bolas_doesnt_sacrifice_a_creature_you_no_longer_control() {
    cr!("701.21a", "603.7b", "613.1b");
    ruling!(
        "Slave of Bolas",
        "If you don’t control the targeted creature when the “at end of turn” ability resolves, it won’t be sacrificed. If it’s still on the battlefield but under another player’s control, the ability won’t try to have you sacrifice it again at the end of the turn after that."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Slave of Bolas", &[&[obj(bears)]]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1 takes it back until end of turn.
    cast(&mut t, P1, "Ray of Command", &[&[obj(bears)]]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    to_end_step(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // At the cleanup, Ray of Command ends: P0 controls it again (Slave of Bolas's
    // effect lasts indefinitely); at the next end step, nothing sacrifices it.
    t.advance_to(P1, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn disruption_aura_mana_cost_includes_colored_mana() {
    cr!("202.1", "118.12");
    ruling!(
        "Disruption Aura",
        "Mana cost includes any requirements for colored mana, although most artifacts don’t have any color requirements."
    );
    supported("Disruption Aura");
    // Master of Etherium costs {2}{U}: three colorless mana can't pay it.
    for blue in [false, true] {
        let mut t = TestGame::new(2);
        let master = t.battlefield(P1, "Master of Etherium");
        let aura = t.battlefield(P0, "Disruption Aura");
        assert!(t.g.attach(aura, obj(master)));
        to_upkeep(&mut t, P1);
        assert_eq!(t.stack_len(), 1);
        let pool = &mut t.g.players[1].mana_pool;
        pool.add_type(if blue { ManaType::U } else { ManaType::C }, 1);
        pool.add_type(ManaType::C, 2);
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(master), blue, "blue {blue}");
    }
}

#[test]
fn sheoldred_transforms_even_if_the_graveyards_shrink() {
    cr!("602.5", "602.2");
    ruling!(
        "Sheoldred // The True Scriptures",
        "Once you’ve activated Sheoldred’s last ability, it doesn’t matter what happens to the number of cards in opponents’ graveyards. The ability will resolve even if no opponent has eight or more cards in their graveyard by that time."
    );
    let name = "Sheoldred // The True Scriptures";
    supported(name);
    let mut t = TestGame::new(2);
    let sheoldred = t.battlefield(P0, name);
    let cards: Vec<ObjectId> = (0..8).map(|_| t.graveyard(P1, "Grizzly Bears")).collect();
    lots(&mut t, P0);
    t.activate(P0, sheoldred, 0, &[]).expect("activated");
    for c in cards {
        move_to(&mut t, c, Zone::Exile);
    }
    t.resolve_all();
    let now = t.named_on_battlefield("The True Scriptures");
    assert_eq!(now.len(), 1);
}

#[test]
fn idol_of_false_gods_dies_with_both_kinds_of_counters() {
    cr!("704.5f", "704.5q", "704.3");
    ruling!(
        "Idol of False Gods",
        "In the rare case where enough -1/-1 counters are placed on Idol of False Gods to reduce its toughness to 0 or less while it's a creature because of the effect of its last ability, Idol of False Gods will be put into its owner's graveyard when state-based actions are performed. Effects that care about the number and types of counters that were on Idol of False Gods when it died will see both the +1/+1 and -1/-1 counters, and any effects that trigger \"whenever a modified creature dies\" will still trigger."
    );
    supported("Idol of False Gods");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian of the Forgotten");
    let idol = t.battlefield(P0, "Idol of False Gods");
    t.g.add_counters(obj(idol), counters::PLUS1, 8, None);
    t.g.recompute();
    assert!(t.obj(idol).is(CardType::Creature));
    t.g.add_counters(obj(idol), counters::MINUS1, 8, None);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(idol));
    // The old object's last known information has both kinds.
    assert_eq!(t.obj(idol).counter(counters::PLUS1), 8);
    assert_eq!(t.obj(idol).counter(counters::MINUS1), 8);
    assert_eq!(triggers_on_stack(&t, "manifest"), 1);
}

#[test]
fn promise_of_loyalty_a_moved_vow_counter_doesnt_restrict_the_new_creature() {
    cr!("122.5", "611.2b");
    ruling!(
        "Promise of Loyalty",
        "If the vow counter is moved to another creature, it won't prevent the second creature from attacking normally."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let vowed = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[obj(mine)]);
    t.answer_choose(P1, &[obj(vowed)]);
    cast(&mut t, P0, "Promise of Loyalty", &[]);
    t.resolve_all();
    let later = t.battlefield(P1, "Hill Giant");
    assert!(!t.g.can_attack_target(vowed, pl(P0)));
    mtg_engine::counter_rules::move_counters(&mut t.g, vowed, obj(later), "vow", 1);
    t.g.recompute();
    assert_eq!(t.counters(later, "vow"), 1);
    assert!(t.g.can_attack_target(later, pl(P0)));
    assert!(t.g.can_attack_target(vowed, pl(P0)));
}

// ---------------------------------------------------------------------------
// Ajani, Nacatl Avenger's reflexive trigger and replacement effects
// ---------------------------------------------------------------------------

#[test]
fn ajani_reflexive_trigger_once_per_cat_warrior_created() {
    cr!("603.12", "614.1a", "111.1");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "In a case where you create more than one Cat Warrior creature token with Ajani, Nacatl Avenger's second ability (for example, because of the effect of Doubling Season), the reflexive ability will trigger once for each Cat Warrior you made this way. If a replacement effect causes you to create different tokens instead (for example, the effect of Divine Visitation), the creation of those tokens won't cause the reflexive ability to trigger."
    );
    interp!("reflexive-trigger-per-specified-token");
    const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";
    for (with, triggers) in [("Doubling Season", 2usize), ("Divine Visitation", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, with);
        t.battlefield(P0, "Raging Goblin");
        let card = t.graveyard(P0, AJANI);
        let ajani = put_onto_battlefield(&mut t, P0, card, true).expect("entered");
        for _ in 0..2 {
            t.answer_targets(P0, &[pl(P1)]);
        }
        t.activate(P0, ajani, 1, &[]).expect("0 ability");
        t.resolve();
        assert_eq!(t.stack_len(), triggers, "{with}");
        t.resolve_all();
        if triggers == 0 {
            assert_eq!(t.life(P1), 20, "{with}");
        } else {
            assert!(t.life(P1) < 20, "{with}");
        }
    }
}

/// The rest of the register entry: "Create two ... tokens. When you do" with Doubling
/// Season (four created: it triggers twice, the provisional part), and "If you do" after a
/// creation Divine Visitation replaced (no Wolf was created: no counter).
#[test]
fn token_creation_counts_the_specified_tokens_for_when_and_if_you_do() {
    cr!("603.12", "603.12a", "614.1a");
    interp!("reflexive-trigger-per-specified-token");
    supported("Faebloom Trick");
    for doubling in [false, true] {
        let mut t = TestGame::new(2);
        if doubling {
            t.battlefield(P0, "Doubling Season");
        }
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(a)]);
        t.answer_targets(P0, &[obj(b)]);
        cast(&mut t, P0, "Faebloom Trick", &[]);
        t.resolve();
        let what = format!("Doubling Season {doubling}");
        assert_eq!(tokens(&t, P0).len(), if doubling { 4 } else { 2 }, "{what}");
        assert_eq!(t.stack_len(), if doubling { 2 } else { 1 }, "{what}");
        t.resolve_all();
        assert!(t.obj_now(a).tapped, "{what}");
        assert_eq!(t.obj_now(b).tapped, doubling, "{what}");
    }
    supported("Turntimber Ranger");
    for visitation in [false, true] {
        let mut t = TestGame::new(2);
        if visitation {
            t.battlefield(P0, "Divine Visitation");
        }
        t.answer_yes(P0, true);
        let ranger = t.enter(P0, "Turntimber Ranger");
        t.resolve_all();
        let what = format!("Divine Visitation {visitation}");
        assert_eq!(tokens(&t, P0).len(), 1, "{what}");
        let angels = t.named_on_battlefield("Angel").len() + t.named_on_battlefield("Angel Token").len();
        assert_eq!(angels, usize::from(visitation), "{what}");
        assert_eq!(t.counters(ranger, counters::PLUS1), u32::from(!visitation), "{what}");
    }
}

// ---------------------------------------------------------------------------
// Friend or foe; modes in order
// ---------------------------------------------------------------------------

#[test]
fn friend_or_foe_triggers_wait_until_the_foes_have_acted() {
    cr!("603.3", "608.2");
    ruling!(
        "Pir's Whim",
        "Players can’t take actions in between the time that friends perform their actions and the time that foes perform theirs. If any abilities trigger from friends performing actions, those triggers won’t be put onto the stack until the foes perform their actions and the spell finishes resolving."
    );
    let mut t = TestGame::new(2);
    // P1's Ankh of Mishra triggers when P0's (friend's) land enters.
    t.battlefield(P1, "Ankh of Mishra");
    let ring = t.battlefield(P1, "Sol Ring");
    let forest = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose(P1, &[obj(ring)]);
    let stack_at_foe = watch(&mut t, P1, is_choose, |g| g.stack.len());
    cast(&mut t, P0, "Pir's Whim", &[]);
    t.resolve();
    assert!(!t.on_battlefield(ring));
    // The foe chose with only the spell on the stack; the Ankh's trigger came after.
    assert_eq!(stack_at_foe.lock().unwrap().first().copied(), Some(1));
    assert_eq!(triggers_on_stack(&t, "damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

/// The same for the other wordings of the friend-or-foe actions (P0 a friend, P1 a foe):
/// what the friends' draws, +1/+1 counters and cards returned from a graveyard trigger goes
/// on the stack only once the foes have acted and the spell has finished resolving.
#[test]
fn friend_or_foe_other_wordings_hold_the_friends_triggers_until_the_foes_have_acted() {
    cr!("603.3", "608.2");
    ruling!(
        "Khorvath's Fury",
        "Players can’t take actions in between the time that friends perform their actions and the time that foes perform theirs."
    );
    ruling!(
        "Regna's Sanction",
        "Players can’t take actions in between the time that friends perform their actions and the time that foes perform theirs."
    );
    ruling!(
        "Virtus's Maneuver",
        "Players can’t take actions in between the time that friends perform their actions and the time that foes perform theirs."
    );
    let friend_and_foe = |t: &mut TestGame| {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
    };
    // Khorvath's Fury: the friend discards one card and draws two, each draw triggering
    // Jace's Erasure (its target chosen as it's put on the stack); the foe is dealt damage
    // equal to the three cards in their hand before that.
    supported("Khorvath's Fury");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace's Erasure");
    t.hand(P0, "Forest");
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    friend_and_foe(&mut t);
    let life_at_target = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.players[1].life,
    );
    cast(&mut t, P0, "Khorvath's Fury", &[]);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(triggers_on_stack(&t, "mill"), 2, "Khorvath's Fury");
    assert_eq!(life_at_target.lock().unwrap().first().copied(), Some(17));
    // Regna's Sanction: the friend's +1/+1 counter triggers Simic Ascendancy; the foe
    // chooses the creature it keeps untapped with only the spell on the stack.
    supported("Regna's Sanction");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Simic Ascendancy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let giant = t.battlefield(P1, "Hill Giant");
    friend_and_foe(&mut t);
    t.answer_choose(P1, &[obj(lions)]);
    let stack_at_foe = watch(&mut t, P1, is_choose, |g| g.stack.len());
    cast(&mut t, P0, "Regna's Sanction", &[]);
    t.resolve();
    assert_eq!(stack_at_foe.lock().unwrap().first().copied(), Some(1));
    assert_eq!(triggers_on_stack(&t, "growth"), 1, "Regna's Sanction");
    assert!(t.obj_now(giant).tapped && !t.obj_now(lions).tapped);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // Virtus's Maneuver: the friend's creature card leaving the graveyard triggers
    // Desecrated Tomb; the foe chooses what to sacrifice with only the spell on the stack.
    supported("Virtus's Maneuver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Desecrated Tomb");
    t.graveyard(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Hill Giant");
    friend_and_foe(&mut t);
    t.answer_choose(P1, &[obj(lions)]);
    let stack_at_foe = watch(&mut t, P1, is_choose, |g| g.stack.len());
    cast(&mut t, P0, "Virtus's Maneuver", &[]);
    t.resolve();
    assert_eq!(stack_at_foe.lock().unwrap().first().copied(), Some(1));
    assert!(!t.on_battlefield(lions) && t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(triggers_on_stack(&t, "Bat"), 1, "Virtus's Maneuver");
}

#[test]
fn rush_of_dread_second_opponent_sees_the_first_sacrifice() {
    cr!("700.2", "608.2c", "702.172a");
    ruling!(
        "Rush of Dread",
        "Like the effects of all modal spells, Rush of Dread’s effects happen in order. If the first two modes target different opponents, the opponent targeted by the second mode will see what the first opponent sacrificed before choosing what to discard."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.hand(P2, "Grizzly Bears");
    t.hand(P2, "Hill Giant");
    let seen = watch(&mut t, P2, is_choose, |g| {
        g.permanents().filter(|o| o.controller == PlayerId(1)).count()
    });
    lots(&mut t, P0);
    let c = t.hand(P0, "Rush of Dread");
    t.cast(P0, c).modes(&[0, 1]).target(pl(P1)).target(pl(P2)).go();
    t.resolve_all();
    assert_eq!(creatures(&t, P1).len(), 1);
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!(seen.lock().unwrap().first().copied(), Some(1));
}

#[test]
fn rootcast_apprenticeship_repeated_modes_follow_the_chosen_order() {
    cr!("700.2d", "700.2a");
    ruling!(
        "Rootcast Apprenticeship",
        "If the same mode is chosen more than once, you choose their relative order as you cast the spell. For example, if you choose the last mode of Rootcast Apprenticeship more than once, you choose the relative order in which the target opponents sacrifice nontoken artifacts."
    );
    for first in [P1, P2] {
        let second = if first == P1 { P2 } else { P1 };
        let mut t = TestGame::new(3);
        for p in [P1, P2] {
            t.battlefield(p, "Sol Ring");
            t.battlefield(p, "Ornithopter");
        }
        let from = t.asked().len();
        lots(&mut t, P0);
        let c = t.hand(P0, "Rootcast Apprenticeship");
        t.cast(P0, c)
            .modes(&[2, 3, 3])
            .target(pl(P0))
            .target(pl(first))
            .target(pl(second))
            .go();
        t.resolve_all();
        let order: Vec<PlayerId> = choosers_since(&t, from)
            .into_iter()
            .filter(|p| *p != P0)
            .collect();
        assert_eq!(order, vec![first, second], "{first:?} first");
        assert_eq!(permanents_of(&t, P1).len(), 1);
        assert_eq!(permanents_of(&t, P2).len(), 1);
        assert_eq!(tokens(&t, P0).len(), 1);
    }
}

// ---------------------------------------------------------------------------
// Spawnsire of Ulamog: casting from outside the game
// ---------------------------------------------------------------------------

#[test]
fn spawnsire_casts_from_the_sideboard_without_alternative_costs() {
    cr!("400.11a", "601.2b", "118.9a", "118.9d");
    ruling!(
        "Spawnsire of Ulamog",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. On the other hand, if the card has additional costs, you may pay those."
    );
    ruling!(
        "Spawnsire of Ulamog",
        "In a casual game, the cards you cast from outside the game come from your personal collection. In a sanctioned event, those cards must come from your sideboard. Note that you can't cast cards from exile this way; exile is a zone in the game."
    );
    let mut t = TestGame::new(2);
    let spawnsire = t.battlefield(P0, "Spawnsire of Ulamog");
    // Nulldrifter (evoke, an alternative cost) and Voidpouncer (kicker, an additional
    // cost) outside the game; another Nulldrifter in exile.
    let drifter = place(&mut t, P0, "Nulldrifter", Zone::Outside(P0));
    let pouncer = place(&mut t, P0, "Voidpouncer", Zone::Outside(P0));
    let exiled = t.exile(P0, "Nulldrifter");
    lots(&mut t, P0);
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(drifter), obj(pouncer)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let from = t.asked().len();
    t.activate(P0, spawnsire, 1, &[]).expect("{20}");
    // Mana for the kicker {2}{C}.
    t.g.players[0].mana_pool.add_type(ManaType::C, 3);
    t.resolve_all();
    let offered = first_offer(&t, P0, from);
    assert!(offered.contains(&drifter) && offered.contains(&pouncer));
    assert!(!offered.contains(&exiled), "exile is a zone in the game");
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Nulldrifter wasn't evoked: it stays.
    assert!(t.on_battlefield(drifter));
    assert!(asked_since(&t, from).iter().all(|(_, d)| match d {
        Decision::ChooseCastingMethod { .. } => !format!("{d:?}").contains("Evoke"),
        _ => true,
    }));
    // Voidpouncer was kicked.
    assert!(t.on_battlefield(pouncer));
    assert_eq!(t.counters(pouncer, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------
// Multiplayer: a player leaves the game (CR 800.4)
// ---------------------------------------------------------------------------

#[test]
fn leaving_the_game_takes_owned_cards_and_ends_control_effects() {
    cr!("800.4a");
    ruling!(
        "Volatile Stormdrake",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the effect giving you control of the target creature ends."
    );
    ruling!(
        "The Eldest Reborn",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the permanent you control from The Eldest Reborn's final chapter ability is exiled."
    );
    supported("Volatile Stormdrake");
    supported("The Eldest Reborn");
    // Volatile Stormdrake: P0 and P1 exchange Stormdrake and a Hill Giant; P0 pays the
    // energy. P0 leaves: the Stormdrake (P0's card) leaves, the Giant returns to P1.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    let drake = t.enter(P0, "Volatile Stormdrake");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(drake).controller, P1);
    t.g.lose_game(P0);
    t.settle();
    assert!(!t.on_battlefield(drake));
    assert_eq!(t.obj_now(giant).controller, P1);
    // The Eldest Reborn: chapter III puts P1's Hill Giant onto the battlefield under P0's
    // control. P0 leaves: it's exiled.
    let mut t = TestGame::new(3);
    let saga = t.battlefield(P0, "The Eldest Reborn");
    t.g.add_counters(obj(saga), "lore", 2, None);
    let card = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(card)]);
    t.g.add_counters(obj(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(controls(&t, P0, card));
    t.g.lose_game(P0);
    t.settle();
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn single_combat_lasts_until_the_left_players_turn_would_have_begun() {
    cr!("800.4m", "611.2a");
    ruling!(
        "Single Combat",
        "In a multiplayer game, if you leave the game after Single Combat resolves but before your next turn begins, its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    supported("Single Combat");
    let mut t = TestGame::new(3);
    cast(&mut t, P0, "Single Combat", &[]);
    t.resolve_all();
    t.g.lose_game(P0);
    t.settle();
    // P1's turn: still can't cast creature spells.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(try_cast(&mut t, P1, "Grizzly Bears", &[]).is_err());
    // P2's turn, then P1's again: P0's turn would have begun in between.
    t.advance_to(P2, Step::PrecombatMain);
    assert!(try_cast(&mut t, P2, "Grizzly Bears", &[]).is_err());
    t.advance_to(P1, Step::PrecombatMain);
    assert!(try_cast(&mut t, P1, "Grizzly Bears", &[]).is_ok());
}

// ---------------------------------------------------------------------------
// "Can't cause you to sacrifice" against an opponent's "unless [a player] sacrifices"
// ---------------------------------------------------------------------------

/// An opponent's spell or ability that has something happen unless a player sacrifices a
/// permanent (or discards a card) can't have a player under Sigarda, Host of Herons, Tajuru
/// Preserver or Tamiyo, Collector of Tales do so, also when that player isn't the one the
/// rest of it is about: Mogis deals the damage, Ogre Marauder gains the evasion, Acererak's
/// controller creates the token (planned with the other opponents' choices) and Torment of
/// Hailfire's opponent loses the life. (Regression: the cost was attributed to the player
/// paying it, not to the resolving ability's controller.)
#[test]
fn an_opponents_unless_sacrifice_cost_cant_be_paid_under_sigarda() {
    cr!("118.12", "701.21a", "701.9a");
    ruling!("Sigarda, Host of Herons", "you can't choose to sacrifice a permanent");
    ruling!(
        "Tajuru Preserver",
        "you can’t choose to sacrifice a permanent. You must perform the action"
    );
    ruling!(
        "Tamiyo, Collector of Tales",
        "you can’t choose to sacrifice or discard"
    );
    for name in [
        "Mogis, God of Slaughter",
        "Ogre Marauder",
        "Acererak the Archlich",
        "Torment of Hailfire",
    ] {
        supported(name);
    }
    for protected in [false, true] {
        let what = |card: &str| format!("{card}, protected {protected}");
        // "At the beginning of each opponent's upkeep, Mogis deals 2 damage to that player
        // unless they sacrifice a creature of their choice."
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mogis, God of Slaughter");
        if protected {
            t.battlefield(P1, "Sigarda, Host of Herons");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[obj(bears)]);
        to_upkeep(&mut t, P1);
        t.resolve_all();
        let expected = if protected { (true, 18) } else { (false, 20) };
        assert_eq!((t.on_battlefield(bears), t.life(P1)), expected, "{}", what("Mogis"));

        // "Whenever Ogre Marauder attacks, it gains 'can't be blocked' until end of turn
        // unless defending player sacrifices a creature of their choice."
        let mut t = TestGame::new(2);
        let ogre = t.battlefield(P0, "Ogre Marauder");
        if protected {
            t.battlefield(P1, "Tajuru Preserver");
        }
        let b1 = t.battlefield(P1, "Grizzly Bears");
        let b2 = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[obj(b1)]);
        attack_with(&mut t, &[(ogre, pl(P1))]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(b1), protected, "{}", what("Ogre Marauder"));
        assert_eq!(t.g.can_block(b2, ogre), !protected, "{}", what("Ogre Marauder"));

        // "For each opponent, you create a 2/2 black Zombie creature token unless that
        // player sacrifices a creature of their choice": P1 (protected) and P2 both try.
        let mut t = TestGame::new(3);
        let acererak = t.battlefield(P0, "Acererak the Archlich");
        if protected {
            t.battlefield(P1, "Sigarda, Host of Herons");
        }
        let mine = t.battlefield(P1, "Grizzly Bears");
        let theirs = t.battlefield(P2, "Grizzly Bears");
        for (p, b) in [(P1, mine), (P2, theirs)] {
            t.answer_yes(p, true);
            t.answer_choose(p, &[obj(b)]);
        }
        attack_with(&mut t, &[(acererak, pl(P1))]);
        t.resolve_all();
        let zombies = tokens(&t, P0).len();
        assert_eq!(t.on_battlefield(mine), protected, "{}", what("Acererak"));
        assert!(!t.on_battlefield(theirs), "{}", what("Acererak"));
        assert_eq!(zombies, usize::from(protected), "{}", what("Acererak"));

        // "Each opponent loses 3 life unless that player sacrifices a nonland permanent of
        // their choice or discards a card", once (X = 1): under Tamiyo P1 can do neither.
        let mut t = TestGame::new(2);
        if protected {
            t.battlefield(P1, "Tamiyo, Collector of Tales");
        }
        let ring = t.battlefield(P1, "Sol Ring");
        t.hand(P1, "Forest");
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[obj(ring)]);
        lots(&mut t, P0);
        let c = t.hand(P0, "Torment of Hailfire");
        t.cast(P0, c).x(1).go();
        t.resolve_all();
        let expected = if protected { (true, 1, 17) } else { (false, 1, 20) };
        assert_eq!(
            (t.on_battlefield(ring), t.hand_size(P1), t.life(P1)),
            expected,
            "{}",
            what("Torment of Hailfire")
        );
    }
    // Sigarda doesn't stop discarding: P1 discards the Forest instead.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sigarda, Host of Herons");
    let ring = t.battlefield(P1, "Sol Ring");
    t.hand(P1, "Forest");
    t.answer_yes(P1, true);
    lots(&mut t, P0);
    let c = t.hand(P0, "Torment of Hailfire");
    t.cast(P0, c).x(1).go();
    t.resolve_all();
    assert_eq!((t.on_battlefield(ring), t.hand_size(P1), t.life(P1)), (true, 0, 20));
}
