//! Rulings batch S26 — Flagbearers and copies: Coalition Honor Guard ("While an opponent is
//! choosing targets as part of casting a spell they control or activating an ability they
//! control, that player must choose at least one Flagbearer on the battlefield if able.")
//! constrains targets chosen while casting or activating (CR 601.2c, 602.2b), not new
//! targets chosen for a copy (CR 707.10c) or changed targets (CR 115.7).

use crate::r_s01_common::supported;
use crate::r_s25_common::{change_copy_targets, targets_of};
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn opponents_must_target_the_honor_guard_while_casting() {
    cr!("601.2c");
    supported("Coalition Honor Guard");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P1, "Coalition Honor Guard");
    t.lands(P0, "Mountain", 1);
    // P0 wants to Shock P1, but must choose the Flagbearer.
    let shock = t.hand(P0, "Shock");
    let shock = t.cast(P0, shock).target(P1).go();
    assert_eq!(targets_of(&t, shock), vec![Entity::Object(guard)]);
    // P1's own spells aren't constrained.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    assert_eq!(targets_of(&t, bolt), vec![Entity::Player(P0)]);
}

#[test]
fn a_copy_with_new_targets_doesnt_have_to_target_a_flagbearer() {
    cr!("601.2c", "707.10c", "115.7");
    ruling!(
        "Coalition Honor Guard",
        "If a spell or ability’s targets are changed, or if a copy of a spell or ability is put onto the stack and has new targets chosen, it doesn’t have to target a Flagbearer."
    );
    supported("Twincast");
    // Twincast: "Copy target instant or sorcery spell. You may choose new targets for the
    // copy."
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P1, "Coalition Honor Guard");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Volcanic Island", 3);
    // The Shock (aimed at the Bears) must target the Honor Guard.
    let shock = t.hand(P0, "Shock");
    let shock = t.cast(P0, shock).target(bears).go();
    assert_eq!(targets_of(&t, shock), vec![Entity::Object(guard)]);
    // Twincast targets a spell, so it can't target a Flagbearer; its copy of the Shock
    // gets a new target: the Bears.
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(shock).go();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_on_stack(&t, copies[0]), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(guard));
    assert_eq!(t.obj_now(guard).damage, 2);
}
