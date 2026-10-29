//! CR 702.59 Recover.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::pay_questions;
use crate::common_k702_052_066::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const RECOVER: &str = "Recover";

/// Destroys all creatures at once.
fn wrath(t: &mut TestGame) {
    run_effect(
        t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: false,
        },
        &[],
    );
}

#[test]
fn recover_returns_the_card_if_its_cost_is_paid() {
    cr!("702.59", "702.59a");
    assert_supported("Grim Harvest");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grim Harvest");
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.settle();
    assert_eq!(stack_triggers(&t, RECOVER).len(), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(pay_questions(&t, P0), 1);
    assert!(t.in_hand(P0, "Grim Harvest"));
    assert!(!t.in_graveyard(P0, "Grim Harvest"));
}

#[test]
fn recover_exiles_the_card_if_its_cost_isnt_paid() {
    cr!("702.59a");
    assert_supported("Resize");
    // Declined.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Resize");
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_exile("Resize"));
    // Can't be paid.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Resize");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(pay_questions(&t, P0), 0);
    assert!(t.in_exile("Resize"));
}

#[test]
fn recover_triggers_only_for_creatures_put_into_your_graveyard() {
    cr!("702.59a");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Resize");
    // An opponent's creature goes to its owner's graveyard.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    t.settle();
    assert!(stack_triggers(&t, RECOVER).is_empty());
    // So does one you control but don't own.
    let borrowed = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[borrowed.0 as usize].base_controller = P0;
    t.g.recompute();
    assert_eq!(t.obj_now(borrowed).controller, P0);
    destroy(&mut t, borrowed);
    t.settle();
    assert!(stack_triggers(&t, RECOVER).is_empty());
    // A noncreature permanent you own doesn't trigger it.
    let anthem = t.battlefield(P0, "Glorious Anthem");
    destroy(&mut t, anthem);
    t.settle();
    assert!(stack_triggers(&t, RECOVER).is_empty());
    // A creature card put into your graveyard from anywhere but the battlefield doesn't.
    let in_hand = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, in_hand, None);
    t.g.flush_events();
    t.settle();
    assert!(stack_triggers(&t, RECOVER).is_empty());
    assert!(t.in_graveyard(P0, "Resize"));
}

#[test]
fn recover_looks_back_so_cards_arriving_with_the_creature_dont_trigger() {
    cr!("702.59a");
    assert_supported("Garza's Assassin");
    let mut t = TestGame::new(2);
    // A creature card with recover dying doesn't trigger its own recover ability, and
    // neither does a creature dying at the same time.
    t.battlefield(P0, "Garza's Assassin");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    t.settle();
    assert!(stack_triggers(&t, RECOVER).is_empty());
    // Once it's in the graveyard, a later creature does.
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.settle();
    assert_eq!(stack_triggers(&t, RECOVER).len(), 1);
}

#[test]
fn a_recover_cost_can_be_paying_half_your_life() {
    cr!("702.59a");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 15;
    t.graveyard(P0, "Garza's Assassin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Garza's Assassin"));
    // Half of 15, rounded up.
    assert_eq!(t.life(P0), 7);
}

#[test]
fn recover_does_nothing_once_the_card_has_left_the_graveyard() {
    cr!("702.59a");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Sun's Bounty");
    t.lands(P0, "Plains", 4);
    // Two creatures die at once: recover triggers twice.
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    t.settle();
    assert_eq!(stack_triggers(&t, RECOVER).len(), 2);
    // The first returns it; the second can't find it (neither pays nor exiles it).
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Sun's Bounty"));
    assert!(!t.in_exile("Sun's Bounty"));
    assert_eq!(pay_questions(&t, P0), 1);
}
