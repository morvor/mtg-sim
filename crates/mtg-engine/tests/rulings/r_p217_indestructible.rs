//! Rulings batch P217 — indestructible (CR 702.12): lethal damage and "destroy" don't put
//! it into the graveyard (CR 702.12b), but other events do.

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Sacrifices the permanent (as its controller would) and settles.
fn sacrifice(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let p = t.obj(id).controller;
    t.g.sacrifice(id, p);
    t.g.flush_events();
    t.settle();
}

#[test]
fn indestructible_darksteel_axe_can_still_be_sacrificed() {
    cr!("702.12b", "701.8b", "701.21a");
    ruling!(
        "Darksteel Axe",
        "Although Darksteel Axe has indestructible, it can still be put into the graveyard for other reasons. The most likely reason is if it's sacrificed."
    );
    supported("Darksteel Axe");
    let mut t = TestGame::new(2);
    let axe = t.battlefield(P0, "Darksteel Axe");
    destroy(&mut t, axe);
    assert!(t.on_battlefield(axe));
    sacrifice(&mut t, axe);
    assert!(t.in_graveyard(P0, "Darksteel Axe"));
}

#[test]
fn indestructible_creatures_still_die_to_sacrifice_the_legend_rule_and_zero_toughness() {
    cr!("702.12b", "704.5f", "704.5g", "704.5j", "701.21a");
    ruling!(
        "Spearbreaker Behemoth",
        "Lethal damage and effects that say “destroy” won't cause a creature with indestructible to be put into the graveyard. However, a creature with indestructible can be put into the graveyard for a number of reasons. The most likely reasons are if it's sacrificed, if it's legendary and another legendary creature with the same name is controlled by the same player, or if its toughness is 0 or less."
    );
    supported("Spearbreaker Behemoth");
    supported("Avacyn, Angel of Hope");
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Spearbreaker Behemoth");
    // Lethal damage and "destroy" don't.
    damage(&mut t, behemoth, 7, behemoth);
    destroy(&mut t, behemoth);
    assert!(t.on_battlefield(behemoth));
    // Toughness 0 or less does.
    modify_until_eot(&mut t, behemoth, vec![Modification::ModifyPT(Value::c(0), Value::c(-5))]);
    assert_eq!(t.zone(behemoth), Zone::Graveyard(P0));
    // Being sacrificed does — a creature given indestructible by its ability ("{1}:
    // Target creature with power 5 or greater gains indestructible until end of turn.").
    let behemoth = t.battlefield(P0, "Spearbreaker Behemoth");
    let giant = t.battlefield(P0, "Craw Wurm");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, behemoth, "gains indestructible").expect("activate");
    t.resolve_all();
    destroy(&mut t, giant);
    assert!(t.on_battlefield(giant));
    sacrifice(&mut t, giant);
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    // The legend rule does: Avacyn, Angel of Hope ("This creature and other permanents
    // you control have indestructible.") — P0 keeps one of two.
    let a = t.battlefield(P0, "Avacyn, Angel of Hope");
    let b = t.battlefield(P0, "Avacyn, Angel of Hope");
    t.g.recompute();
    t.settle();
    let left = [a, b].iter().filter(|x| t.on_battlefield(**x)).count();
    assert_eq!(left, 1);
    assert!(t.in_graveyard(P0, "Avacyn, Angel of Hope"));
}
