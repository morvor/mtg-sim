//! Rulings batch P217 — landfall ("Whenever a land you control enters, ..."; an ability
//! word, CR 207.2c).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::*;

const LANDFALL: &str = "a land you control enters";

#[test]
fn bloodghast_triggers_only_if_already_in_the_graveyard_as_the_land_enters() {
    cr!("603.6a", "603.10", "603.2");
    ruling!(
        "Bloodghast",
        "Bloodghast's landfall ability triggers only if it's already in your graveyard at the time a land enters under your control."
    );
    supported("Bloodghast");
    // On the battlefield as the land enters: it doesn't trigger (its landfall ability
    // works only from the graveyard), even if it dies right after.
    let mut t = TestGame::new(2);
    let ghast = t.battlefield(P0, "Bloodghast");
    enter(&mut t, P0, "Swamp");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 0);
    destroy(&mut t, ghast);
    assert!(t.in_graveyard(P0, "Bloodghast"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bloodghast"));
    // Already in the graveyard: it triggers and may return.
    enter(&mut t, P0, "Swamp");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bloodghast").len(), 1);
    // In hand: no trigger.
    let mut t = TestGame::new(2);
    t.hand(P0, "Bloodghast");
    enter(&mut t, P0, "Swamp");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 0);
}

#[test]
fn adventuring_gear_gone_gives_the_bonus_to_the_creature_it_was_attached_to() {
    cr!("608.2h", "113.7a", "301.5");
    ruling!(
        "Adventuring Gear",
        "If Adventuring Gear leaves the battlefield before its landfall ability resolves, the creature it was attached to at the time it left the battlefield gets +2/+2. If it wasn't attached to a creature at that time, nothing gets the bonus."
    );
    supported("Adventuring Gear");
    // Attached to the Bears as it leaves: the Bears get +2/+2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gear = attach_new(&mut t, P0, "Adventuring Gear", bears);
    enter(&mut t, P0, "Forest");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.g.sacrifice(gear, P0);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Adventuring Gear"));
    assert_eq!(t.pt(bears), (4, 4));
    // Unattached as it leaves (it was moved off the Bears in response): nothing gets it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gear = attach_new(&mut t, P0, "Adventuring Gear", bears);
    enter(&mut t, P0, "Forest");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.g.unattach(gear);
    t.g.recompute();
    t.g.sacrifice(gear, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}
