//! Rulings batch S30 — indestructible (CR 702.12): lethal damage and "destroy" don't put
//! an indestructible permanent into the graveyard, but sacrifice, the legend rule, 0
//! toughness, and 0 loyalty still do.

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::damage;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Checks that `id` survives lethal damage and a "destroy" effect, then dies to 0
/// toughness.
fn survives_damage_and_destroy_but_not_0_toughness(t: &mut TestGame, source: ObjectId, id: ObjectId) {
    let toughness = t.pt(id).1;
    damage(t, source, toughness + 3, id);
    assert!(t.on_battlefield(id));
    assert!(t.obj_now(id).damage >= toughness as u32);
    destroy(t, id);
    assert!(t.on_battlefield(id));
    modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(0), Value::c(-toughness))],
    );
    assert!(!t.on_battlefield(id));
}

#[test]
fn indestructible_ulamog_leaves_by_sacrifice_or_the_legend_rule() {
    cr!("702.12b", "704.5j", "704.5f", "701.21a");
    ruling!(
        "Ulamog, the Infinite Gyre",
        "Lethal damage and effects that say \"destroy\" won't cause a creature with indestructible to be put into the graveyard. However, a creature with indestructible can be put into the graveyard for a number of reasons."
    );
    supported("Ulamog, the Infinite Gyre");
    let mut t = TestGame::new(2);
    let source = t.battlefield(P1, "Grizzly Bears");
    let ulamog = t.battlefield(P0, "Ulamog, the Infinite Gyre");
    survives_damage_and_destroy_but_not_0_toughness(&mut t, source, ulamog);
    // Sacrificed.
    let mut t = TestGame::new(2);
    let ulamog = t.battlefield(P0, "Ulamog, the Infinite Gyre");
    t.g.sacrifice(ulamog, P0);
    t.settle();
    assert!(!t.on_battlefield(ulamog));
    // Two legendary creatures with the same name under one player's control: one goes.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ulamog, the Infinite Gyre");
    let b = t.battlefield(P0, "Ulamog, the Infinite Gyre");
    t.settle();
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1
    );
}

#[test]
fn a_creature_given_indestructible_still_dies_to_0_toughness_or_sacrifice() {
    cr!("702.12b", "704.5f");
    ruling!(
        "Deathless Angel",
        "Lethal damage and effects that say “destroy” won’t cause a creature with indestructible to be put into the graveyard. However, a creature with indestructible can be put into the graveyard for a number of reasons."
    );
    supported("Deathless Angel");
    let mut t = TestGame::new(2);
    // "{W}{W}: Target creature gains indestructible until end of turn."
    let angel = t.battlefield(P0, "Deathless Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    t.activate(P0, angel, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let source = t.battlefield(P1, "Hill Giant");
    survives_damage_and_destroy_but_not_0_toughness(&mut t, source, bears);
    // Sacrificed.
    let lions = t.battlefield(P0, "Savannah Lions");
    t.activate(P0, angel, 0, &[Entity::Object(lions)]).unwrap();
    t.resolve_all();
    destroy(&mut t, lions);
    assert!(t.on_battlefield(lions));
    t.g.sacrifice(lions, P0);
    t.settle();
    assert!(t.in_graveyard(P0, "Savannah Lions"));
}

#[test]
fn an_indestructible_planeswalker_still_loses_loyalty_and_dies_at_0() {
    cr!("120.3c", "704.5i", "702.12b");
    ruling!(
        "Avacyn, Angel of Hope",
        "A planeswalker with indestructible still loses loyalty as it's dealt damage. It is put into its owner's graveyard if its loyalty becomes 0."
    );
    supported("Avacyn, Angel of Hope");
    let mut t = TestGame::new(2);
    // "Other permanents you control have indestructible."
    t.battlefield(P0, "Avacyn, Angel of Hope");
    let jace = t.battlefield(P0, "Jace Beleren");
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 2, jace);
    assert!(t.on_battlefield(jace));
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    damage(&mut t, source, 1, jace);
    assert!(!t.on_battlefield(jace));
    assert!(t.in_graveyard(P0, "Jace Beleren"));
}
