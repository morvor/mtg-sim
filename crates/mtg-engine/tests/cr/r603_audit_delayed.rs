//! CR 603.7 audit: delayed triggered abilities. They act on the object they refer to even
//! after its characteristics change, but not on a new object it became (603.7c); they're
//! controlled by the player who controlled the spell or ability that created them as it
//! resolved, whatever happens to its source (603.7d–e), and that player can only do what
//! the effect lets them (sacrificing only what they control); "at the beginning of the
//! next end step" created during an end step waits for the next turn's (513.2); one with
//! a duration triggers every time until it ends (603.7b) and never on a permanent entering
//! in the state it watches for (603.2e).

use crate::r603_audit_common::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 activates Sneak Attack, putting `name` from their hand onto the battlefield; returns
/// the creature as it is on the battlefield.
fn sneak(t: &mut TestGame, name: &str) -> ObjectId {
    let sa = t.battlefield(P0, "Sneak Attack");
    let card = t.hand(P0, name);
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.activate(P0, sa, 0, &[]).expect("activate Sneak Attack");
    t.resolve();
    t.clear_answers();
    let c = t.g.current(card);
    assert!(t.on_battlefield(c), "{name} was put onto the battlefield");
    c
}

#[test]
fn a_delayed_sacrifice_doesnt_follow_the_creature_to_a_new_object() {
    cr!("603.7c", "400.7");
    ruling!(
        "Sneak Attack",
        "If that creature has left the battlefield, even if it came back, you don't sacrifice it."
    );
    let mut t = TestGame::new(2);
    let giant = sneak(&mut t, "Hill Giant");
    t.lands(P0, "Plains", 1);
    let shift = t.hand(P0, "Cloudshift");
    t.cast(P0, shift).target(giant).go();
    t.resolve_all();
    let giant = t.g.current(giant);
    assert!(t.on_battlefield(giant));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(giant), "the returned Giant is a new object");
}

#[test]
fn a_delayed_sacrifice_follows_the_creature_through_characteristic_changes() {
    cr!("603.7c");
    supported("Imprisoned in the Moon");
    let mut t = TestGame::new(2);
    let giant = sneak(&mut t, "Hill Giant");
    t.lands(P0, "Island", 3);
    let moon = t.hand(P0, "Imprisoned in the Moon");
    t.cast(P0, moon).target(giant).go();
    t.resolve_all();
    assert!(t.obj_now(giant).is(CardType::Land));
    assert!(!t.obj_now(giant).is(CardType::Creature));
    t.advance_to(P1, Step::Upkeep);
    assert!(
        t.in_graveyard(P0, "Hill Giant"),
        "sacrificed though it's no longer a creature"
    );
}

#[test]
fn a_delayed_sacrifice_is_done_by_its_controller_only_if_they_still_control_it() {
    cr!("603.7e", "701.21a");
    ruling!(
        "Sneak Attack",
        "You sacrifice the creature only if you still control it."
    );
    let mut t = TestGame::new(2);
    let giant = sneak(&mut t, "Hill Giant");
    // P1 takes the Giant until end of turn.
    t.lands(P1, "Island", 4);
    let ray = t.hand(P1, "Ray of Command");
    t.cast(P1, ray).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P1);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(giant), "P0 couldn't sacrifice P1's creature");
    assert_eq!(t.obj_now(giant).controller, P0, "back to P0 after the turn");
}

#[test]
fn a_delayed_trigger_happens_after_its_source_left_the_battlefield() {
    cr!("603.7e", "113.7a");
    let mut t = TestGame::new(2);
    let giant = sneak(&mut t, "Hill Giant");
    let sa = t.named_on_battlefield("Sneak Attack")[0];
    t.g.destroy(sa, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sneak Attack"));
    t.advance_to_step(Step::End);
    t.settle();
    let d = stack_triggers(&t);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].1, P0);
    assert_eq!(d[0].2, "Sneak Attack", "its source is the ability's source");
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_spells_delayed_trigger_is_the_spells_and_its_casters() {
    cr!("603.7d");
    supported("Through the Breach");
    let mut t = TestGame::new(2);
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let breach = t.hand(P0, "Through the Breach");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, breach).go();
    t.resolve_all();
    let giant = t.g.current(giant);
    assert!(t.on_battlefield(giant));
    t.advance_to_step(Step::End);
    t.settle();
    let d = stack_triggers(&t);
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].1, d[0].2.as_str()), (P0, "Through the Breach"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_delayed_return_happens_without_its_source_but_not_for_a_token() {
    cr!("603.7e", "111.7");
    ruling!(
        "Flickerwisp",
        "The exiled card will return to the battlefield at the beginning of the end step even if Flickerwisp is no longer on the battlefield."
    );
    ruling!(
        "Flickerwisp",
        "If a token is exiled this way, it will cease to exist and won't return to the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let wisp = t.hand(P0, "Flickerwisp");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, wisp).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.destroy(t.g.current(wisp), None);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1, "under its owner's control");
    // A token doesn't come back.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 7);
    let spirits = t.hand(P0, "Raise the Alarm");
    t.cast(P0, spirits).go();
    t.resolve_all();
    let soldier = t.named_on_battlefield("Soldier Token")[0];
    let wisp = t.hand(P0, "Flickerwisp");
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    t.cast(P0, wisp).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 1);
}

#[test]
fn a_permanent_returned_at_the_end_step_misses_that_end_steps_triggers() {
    cr!("603.7", "513.2");
    ruling!(
        "Flickerwisp",
        "If the permanent that returns to the battlefield has any abilities that trigger at the beginning of the end step, those abilities won't trigger that turn."
    );
    let mut t = TestGame::new(2);
    let ball = t.battlefield(P0, "Ball Lightning");
    t.lands(P0, "Plains", 3);
    let wisp = t.hand(P0, "Flickerwisp");
    t.answer_targets(P0, &[Entity::Object(ball)]);
    t.cast(P0, wisp).go();
    t.resolve_all();
    assert_eq!(t.zone(ball), Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    let ball = t.g.current(ball);
    assert!(
        t.on_battlefield(ball),
        "returned at P0's end step, it wasn't sacrificed then"
    );
    // At P1's end step it triggers ("at the beginning of the end step": every turn).
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ball Lightning"));
}

#[test]
fn a_next_end_step_delayed_trigger_created_during_an_end_step_waits_a_turn() {
    cr!("603.7", "513.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Flickerwisp");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(
        t.zone(bears),
        Zone::Exile,
        "not this turn's end step, which had begun"
    );
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.on_battlefield(t.g.current(bears)), "the next turn's");
}

#[test]
fn a_delayed_trigger_with_a_duration_triggers_each_time_but_not_on_entering_tapped() {
    cr!("603.7b", "603.2e");
    ruling!(
        "Don't Move",
        "Don't Move won't affect a creature that enters the battlefield tapped."
    );
    supported("Don't Move");
    supported("Dread Wanderer");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(tapped);
    let p1_giant = t.battlefield(P1, "Hill Giant");
    let p0_bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let dm = t.hand(P0, "Don't Move");
    t.cast(P0, dm).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the tapped creature");
    assert!(t.on_battlefield(p1_giant) && t.on_battlefield(p0_bears));
    // A creature entering tapped didn't become tapped.
    let wanderer = t.enter(P0, "Dread Wanderer");
    t.resolve_all();
    assert!(t.obj_now(wanderer).tapped);
    assert!(t.on_battlefield(wanderer));
    // Each creature becoming tapped until P0's next turn is destroyed, on P1's turn too.
    t.g.tap(p0_bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.advance_to(P1, Step::PrecombatMain);
    t.g.tap(p1_giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // It ended with P0's next turn.
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_lose_control_delayed_trigger_happens_in_the_cleanup_step() {
    cr!("603.7", "603.10d", "514.3a");
    ruling!(
        "Ray of Command",
        "It taps during the Cleanup step (if it is not already tapped), so any abilities triggered off it tapping happen at that time."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let ray = t.hand(P0, "Ray of Command");
    t.cast(P0, ray).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(!t.obj_now(bears).tapped);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::Cleanup && !g.stack.is_empty() || g.turn.active == P1
    });
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::Cleanup);
    assert_eq!(t.obj_now(bears).controller, P1);
    let d = stack_triggers(&t);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].1, P0, "the delayed trigger is the caster's");
    t.resolve();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn a_this_turn_delayed_trigger_doesnt_trigger_on_a_later_turn() {
    cr!("603.7b");
    ruling!(
        "Cryptek",
        "If the target artifact creature isn't put into a graveyard this turn, that ability never triggers."
    );
    let mut t = TestGame::new(2);
    let cryptek = t.battlefield(P0, "Cryptek");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Swamp", 2);
    t.activate(P0, cryptek, 0, &[Entity::Object(thopter)])
        .expect("activate Cryptek");
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    t.g.destroy(thopter, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Cryptek"), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn a_delayed_sacrifice_cant_affect_a_phased_out_permanent() {
    cr!("603.7c", "702.26b");
    let mut t = TestGame::new(2);
    let giant = sneak(&mut t, "Hill Giant");
    t.lands(P0, "Island", 1);
    let slip = t.hand(P0, "Slip Out the Back");
    t.cast(P0, slip).target(giant).go();
    t.resolve_all();
    assert!(t.obj_now(giant).phased_out);
    // At the end step the delayed ability resolves and can't find it; it doesn't wait.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(giant));
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(giant).phased_out);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(giant), "never sacrificed");
}
