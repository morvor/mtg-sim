//! Rulings on survival ("At the beginning of your second main phase, if this creature is
//! tapped, ..."): the intervening "if" is checked when it would trigger and again as it
//! resolves (CR 603.4), with last known information if the creature is gone.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s16_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Glimmer Seeker (3/3): "Survival — At the beginning of your second main phase, if this
/// creature is tapped, draw a card if you control a Glimmer creature. If you don't control
/// a Glimmer creature, create a 1/1 white Glimmer enchantment creature token."
const SEEKER: &str = "Glimmer Seeker";

fn glimmers(t: &TestGame) -> usize {
    with_subtype(t, P0, "Glimmer").len()
}

fn tapped_seeker(t: &mut TestGame) -> ObjectId {
    supported(SEEKER);
    let s = t.battlefield(P0, SEEKER);
    t.g.tap(s);
    t.g.flush_events();
    s
}

#[test]
fn a_survival_ability_does_nothing_if_the_creature_is_untapped_as_it_resolves() {
    cr!("603.4");
    ruling!(
        "Glimmer Seeker",
        "If a creature's survival ability triggers but that creature is untapped when the ability begins to resolve, that ability won't do anything."
    );
    let mut t = TestGame::new(2);
    let s = tapped_seeker(&mut t);
    to_second_main(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "second main phase"), 1);
    // In response, it untaps.
    t.g.untap(s);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(glimmers(&t), 0);

    // Still tapped: it creates a Glimmer.
    let mut t = TestGame::new(2);
    tapped_seeker(&mut t);
    to_second_main(&mut t, P0);
    t.resolve_all();
    assert_eq!(glimmers(&t), 1);
}

#[test]
fn a_survival_ability_doesnt_trigger_if_the_creature_is_untapped_as_the_phase_begins() {
    cr!("603.4", "505.1");
    ruling!(
        "Glimmer Seeker",
        "If a creature with a survival ability isn't tapped when your second main phase begins, the ability won't trigger at all. You won't be able to tap it during your second main phase in time to have that ability trigger."
    );
    supported(SEEKER);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, SEEKER);
    to_second_main(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "second main phase"), 0);
    // Tapping it during the second main phase is too late.
    t.g.tap(s);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(glimmers(&t), 0);
}

#[test]
fn a_survival_ability_uses_the_last_known_tapped_status_of_a_creature_that_left() {
    cr!("603.4", "608.2h");
    ruling!(
        "Glimmer Seeker",
        "If a creature's survival ability triggers but the creature leaves the battlefield before the ability resolves, use its tapped or untapped status as it last existed on the battlefield to determine whether or not the ability will do anything."
    );
    // It leaves the battlefield tapped: the ability still creates a Glimmer.
    let mut t = TestGame::new(2);
    let s = tapped_seeker(&mut t);
    to_second_main(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "second main phase"), 1);
    destroy(&mut t, s);
    assert!(t.in_graveyard(P0, SEEKER));
    t.resolve_all();
    assert_eq!(glimmers(&t), 1);

    // It's untapped, then leaves: the ability does nothing.
    let mut t = TestGame::new(2);
    let s = tapped_seeker(&mut t);
    to_second_main(&mut t, P0);
    t.g.untap(s);
    t.g.flush_events();
    destroy(&mut t, s);
    t.resolve_all();
    assert_eq!(glimmers(&t), 0);
}
