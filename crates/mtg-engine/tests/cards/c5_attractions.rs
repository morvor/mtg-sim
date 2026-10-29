//! Attractions (cards5-game-rules-dice, `attractions`): a duration ending when any player
//! rolls a 1 (Drop Tower), phasing out until you roll a 3 or less while rolling to visit
//! and "that hasn't been phased out with this Attraction" (Ferris Wheel), a visit trigger
//! created for a chosen Attraction (Myra the Magnificent), and a reveal trigger comparing
//! the card with the roll's result (Priority Boarding). Die results are loaded.

use crate::combat2_common::supported;
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` rolls to visit their Attractions with the given die result.
fn roll_to_visit(t: &mut TestGame, p: PlayerId, result: u32) {
    t.g.dice.loaded.push_back(result);
    mtg_engine::variants::roll_to_visit(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

/// `p` rolls a d20 for an effect (not to visit), with the given result.
fn roll_d20(t: &mut TestGame, p: PlayerId, result: u32) {
    let src = t.battlefield(p, "Memnite");
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src), p);
    t.g.dice.loaded.push_back(result);
    mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(20), &mut ctx);
    t.g.flush_events();
    t.settle();
}

fn flying(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.has_keyword(KeywordKind::Flying)
}

// ---------------------------------------------------------------------------
// Drop Tower
// ---------------------------------------------------------------------------

/// Drop Tower (lit up 2, 3, 6) visited: the Bears gain flying.
fn drop_tower() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drop Tower");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    roll_to_visit(&mut t, P0, 3);
    t.resolve_all();
    assert!(flying(&t, bears));
    (t, bears)
}

#[test]
fn drop_tower_flying_ends_when_any_player_rolls_a_1() {
    cr!("611.2a", "706.2", "702.159a");
    supported("Drop Tower");
    ruling!(
        "Drop Tower",
        "As soon as a player rolls a 1, the creature will no longer be granted flying from Drop Tower."
    );
    let (mut t, bears) = drop_tower();
    // Other results don't end it.
    roll_d20(&mut t, P1, 2);
    assert!(flying(&t, bears));
    // An opponent's 1 does, as the result is known.
    roll_d20(&mut t, P1, 1);
    assert!(!flying(&t, bears));
}

#[test]
fn drop_tower_flying_otherwise_lasts_until_end_of_turn() {
    cr!("611.2a", "514.2");
    let (mut t, bears) = drop_tower();
    t.advance_to(P0, Step::End);
    assert!(flying(&t, bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(!flying(&t, bears));
    // A 1 rolled later ends nothing (the effect is gone).
    roll_d20(&mut t, P0, 1);
    assert!(!flying(&t, bears));
}

// ---------------------------------------------------------------------------
// Ferris Wheel
// ---------------------------------------------------------------------------

#[test]
fn ferris_wheel_phases_out_until_you_roll_a_3_or_less_to_visit() {
    cr!("610.4", "702.26b", "701.52a");
    supported("Ferris Wheel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ferris Wheel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    roll_to_visit(&mut t, P0, 5);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    // Not an untap step, an opponent's roll to visit, or a die roll of yours for another
    // reason.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out);
    roll_to_visit(&mut t, P1, 2);
    roll_d20(&mut t, P0, 2);
    assert!(t.obj_now(bears).phased_out);
    // Your roll to visit with a 4: no.
    t.answer_targets(P0, &[]);
    roll_to_visit(&mut t, P0, 4);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    roll_to_visit(&mut t, P0, 3);
    assert!(!t.obj_now(bears).phased_out);
}

#[test]
fn ferris_wheel_cant_target_a_creature_it_phased_out_before() {
    cr!("115.1");
    ruling!(
        "Ferris Wheel",
        "Each Ferris Wheel that comes under your control is a new, exciting experience."
    );
    let mut t = TestGame::new(2);
    let wheel = t.battlefield(P0, "Ferris Wheel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    roll_to_visit(&mut t, P0, 1);
    assert!(!t.obj_now(bears).phased_out);
    // The next visit: only the Giant can be targeted.
    t.answer_targets(P0, &[Entity::Object(giant)]);
    roll_to_visit(&mut t, P0, 6);
    let offered: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    let last = offered.last().expect("targets were chosen");
    assert!(last.contains(&Entity::Object(giant)));
    assert!(!last.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.obj_now(giant).phased_out);
    // Another Ferris Wheel may target the Bears.
    let other = t.battlefield(P0, "Ferris Wheel");
    assert_ne!(other, wheel);
    roll_to_visit(&mut t, P0, 1);
    assert!(!t.obj_now(giant).phased_out);
    // Both are visited: the first one has no legal target (it phased both out before).
    t.answer_targets(P0, &[Entity::Object(bears)]);
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
}

// ---------------------------------------------------------------------------
// Myra the Magnificent
// ---------------------------------------------------------------------------

/// Myra activated for Divination (X = 3) with Drop Tower (2, 3, 6) and Ferris Wheel (4, 5,
/// 6) on the battlefield; the Drop Tower is chosen.
fn myra() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let myra = t.battlefield(P0, "Myra the Magnificent");
    let tower = t.battlefield(P0, "Drop Tower");
    t.battlefield(P0, "Ferris Wheel");
    let div = t.graveyard(P0, "Divination");
    mana(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(P0, &[Entity::Object(tower)]);
    t.activate(P0, myra, 0, &[Entity::Object(div)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Divination"));
    assert_eq!(t.counters(tower, "midway"), 1);
    (t, myra, tower)
}

#[test]
fn myra_copies_the_exiled_card_whenever_you_visit_the_chosen_attraction() {
    cr!("603.7a", "701.52a", "707.12");
    supported("Myra the Magnificent");
    ruling!(
        "Myra the Magnificent",
        "This ability will continue to trigger as long as the Attraction is on the battlefield and the exiled card remains exiled, even if Myra leaves the battlefield"
    );
    let (mut t, myra, _tower) = myra();
    // Myra leaving doesn't matter.
    t.g.move_object(myra, mtg_engine::object::Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    for _ in 0..2 {
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        // (Drop Tower's own visit ability needs a target: none.)
        roll_to_visit(&mut t, P0, 2);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 2, "the copy of Divination was cast");
    }
    assert!(t.in_exile("Divination"));
}

#[test]
fn myra_visits_of_another_attraction_dont_copy_it_and_it_ends_when_the_attraction_leaves() {
    cr!("603.7a", "611.2b");
    let (mut t, _myra, tower) = myra();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    roll_to_visit(&mut t, P0, 5);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "the Ferris Wheel was visited, not the Drop Tower");
    t.g.move_object(tower, mtg_engine::object::Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    // Another Drop Tower (a new object) visited: no copy.
    t.battlefield(P0, "Drop Tower");
    t.answer_yes(P0, true);
    roll_to_visit(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn myra_declining_to_cast_the_copy() {
    cr!("603.7a", "608.2d");
    let (mut t, _myra, _tower) = myra();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    roll_to_visit(&mut t, P0, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

// ---------------------------------------------------------------------------
// Priority Boarding
// ---------------------------------------------------------------------------

/// Priority Boarding with Hill Giant (mana value 4) on top of P0's library; P0 rolls a
/// d20 showing `result` and answers yes to every "may".
fn boarding(result: u32) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Priority Boarding");
    let giant = t.library_top(P0, "Hill Giant");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    roll_d20(&mut t, P0, result);
    t.resolve_all();
    (t, giant)
}

#[test]
fn priority_boarding_exiles_a_card_with_lesser_mana_value_and_you_may_play_it() {
    cr!("603.2", "603.12", "706.2");
    supported("Priority Boarding");
    let (mut t, giant) = boarding(5);
    assert!(t.in_exile("Hill Giant"));
    let g = t.g.current(giant);
    mana(&mut t, P0);
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn priority_boarding_doesnt_exile_a_card_with_equal_or_greater_mana_value() {
    cr!("603.2");
    ruling!(
        "Priority Boarding",
        "This is true even if the card you reveal has a mana value too great to be exiled"
    );
    let (mut t, _giant) = boarding(4);
    assert!(!t.in_exile("Hill Giant"));
    assert_eq!(t.obj_now(*t.g.player(P0).library.last().unwrap()).chars.name, "Hill Giant");
    // Once a card was revealed, it doesn't trigger again this turn.
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    roll_d20(&mut t, P0, 20);
    t.resolve_all();
    assert!(!t.in_exile("Hill Giant"));
}

#[test]
fn priority_boarding_does_nothing_more_if_it_left_the_battlefield_before_the_reveal() {
    cr!("603.2");
    interp!("linked-reveal-trigger-as-reflexive");
    let mut t = TestGame::new(2);
    let pb = t.battlefield(P0, "Priority Boarding");
    t.library_top(P0, "Hill Giant");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let src = t.battlefield(P0, "Memnite");
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src), P0);
    t.g.dice.loaded.push_back(10);
    mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(20), &mut ctx);
    t.g.flush_events();
    t.settle();
    // The trigger is on the stack; Priority Boarding leaves before it resolves.
    t.g.move_object(pb, mtg_engine::object::Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(!t.in_exile("Hill Giant"));
}

#[test]
fn ferris_wheel_can_target_a_creature_again_once_it_is_a_new_object() {
    cr!("400.7", "115.1");
    ruling!(
        "Ferris Wheel",
        "it will be a new object with no memory of being phased out by the Ferris Wheel. It is again a legal target."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ferris Wheel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    roll_to_visit(&mut t, P0, 1);
    assert!(!t.obj_now(bears).phased_out);
    // It leaves and returns: a new object.
    t.g.move_object(bears, mtg_engine::object::Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    let back = t.g.current(bears);
    t.g.move_object(back, mtg_engine::object::Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P1));
    let back = t.g.current(back);
    assert_ne!(back, bears);
    t.answer_targets(P0, &[Entity::Object(back)]);
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    assert!(t.obj_now(back).phased_out);
}

#[test]
fn myra_stops_copying_once_the_exiled_card_leaves_exile() {
    cr!("611.2b", "603.7a");
    ruling!(
        "Myra the Magnificent",
        "This ability will continue to trigger as long as the Attraction is on the battlefield and the exiled card remains exiled"
    );
    let (mut t, _myra, _tower) = myra();
    let div = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Divination")[0];
    t.g.move_object(div, mtg_engine::object::Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    roll_to_visit(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "no copy");
}

#[test]
fn priority_boarding_triggers_again_if_you_didnt_reveal() {
    cr!("603.2h");
    ruling!(
        "Priority Boarding",
        "Once you choose to reveal the top card of your library during the resolution of Priority Boarding"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Priority Boarding");
    t.library_top(P0, "Hill Giant");
    // The first time: declined.
    t.answer_yes(P0, false);
    roll_d20(&mut t, P0, 12);
    t.resolve_all();
    assert!(!t.in_exile("Hill Giant"));
    // The second roll triggers it again.
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    roll_d20(&mut t, P0, 12);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
}
