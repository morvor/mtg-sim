//! Rulings batch S27 — the untap symbol {Q} (CR 107.6): "Untap this permanent" is a cost,
//! so an untapped permanent can't activate the ability, and the untapping happens as the
//! ability is activated, with no chance to respond to it (CR 602.2b).

use crate::r_s01_common::*;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s27_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether the permanent (followed across zone changes) is tapped.
fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn patrol_signaler_can_t_activate_its_untap_ability_while_untapped() {
    cr!("107.6", "602.2b");
    ruling!(
        "Patrol Signaler",
        "If the permanent is already untapped, you can’t activate its {Q} ability. That’s because you can’t pay the “Untap this permanent” cost."
    );
    supported("Patrol Signaler");
    // "{1}{W}, {Q}: Create a 1/1 white Kithkin Soldier creature token."
    let mut t = TestGame::new(2);
    let signaler = t.battlefield(P0, "Patrol Signaler");
    t.lands(P0, "Plains", 2);
    assert!(!can_activate_containing(&mut t, P0, signaler, "Create"));
    assert!(activate_containing(&mut t, P0, signaler, "Create").is_err());
    assert_eq!(tapped_lands(&t, P0), 0);
    // Tapped, it can.
    t.g.objects[signaler.0 as usize].tapped = true;
    assert!(can_activate_containing(&mut t, P0, signaler, "Create"));
}

#[test]
fn patrol_signaler_untaps_as_a_cost_and_the_ability_can_be_responded_to() {
    cr!("107.6", "602.2b", "601.2h", "113.7a");
    ruling!(
        "Patrol Signaler",
        "When you activate an {Q} ability, you untap the creature with that ability as a cost. The untap can’t be responded to. (The actual ability can be responded to, of course.)"
    );
    supported("Patrol Signaler");
    let mut t = TestGame::new(2);
    let signaler = t.battlefield(P0, "Patrol Signaler");
    t.g.objects[signaler.0 as usize].tapped = true;
    t.lands(P0, "Plains", 2);
    let ability = activate_containing(&mut t, P0, signaler, "Create")
        .unwrap()
        .expect("on the stack");
    // The Signaler is already untapped while the ability waits on the stack.
    assert!(!tapped(&t, signaler));
    assert_eq!(t.g.stack.last(), Some(&ability));
    // P1 responds by killing it; the ability still resolves.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(signaler).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Patrol Signaler"));
    assert_eq!(with_subtype(&t, P0, "Kithkin").len(), 1);
}

#[test]
fn umbral_mantle_s_untap_ability_needs_a_tapped_creature_and_untaps_it_as_a_cost() {
    cr!("107.6", "602.2b");
    ruling!(
        "Umbral Mantle",
        "If the permanent is already untapped, you can't activate its {Q} ability. That's because you can't pay the \"Untap this permanent\" cost."
    );
    ruling!(
        "Umbral Mantle",
        "When you activate an {Q} ability, you untap the creature with that ability as a cost. The untap can't be responded to. (The actual ability can be responded to, of course.)"
    );
    supported("Umbral Mantle");
    // "Equipped creature has "{3}, {Q}: This creature gets +2/+2 until end of turn.""
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Umbral Mantle", bears);
    t.lands(P0, "Wastes", 6);
    assert!(!can_activate_containing(&mut t, P0, bears, "+2/+2"));
    t.g.objects[bears.0 as usize].tapped = true;
    activate_containing(&mut t, P0, bears, "+2/+2").unwrap();
    assert!(!tapped(&t, bears));
    assert_eq!(t.pt(bears), (2, 2));
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // Untapped now: it can't be activated again until it's tapped.
    assert!(!can_activate_containing(&mut t, P0, bears, "+2/+2"));
}
