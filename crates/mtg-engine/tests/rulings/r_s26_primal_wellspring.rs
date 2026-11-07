//! Rulings batch S26 — Primal Amulet // Primal Wellspring: the Amulet transforms once it
//! has four charge counters (checked as its triggered ability resolves, CR 608.2c,
//! 701.27a), and the Wellspring's mana copies the spell it's spent on (CR 707.10).

use crate::r_s01_common::supported;
use crate::r_s16_common::add_charge;
use crate::r_s17_common::{enter_transformed, face};
use crate::r_s20_common::tap_for_mana;
use crate::r_s26_common::*;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The Elemental tokens Young Pyromancer made for `p`.
fn elementals(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == "Elemental Token")
        .count()
}

#[test]
fn primal_amulet_transforms_with_four_charge_counters() {
    cr!("608.2c", "701.27a", "122.1");
    supported("Primal Amulet // Primal Wellspring");
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Primal Amulet // Primal Wellspring");
    add_charge(&mut t, amulet, 2);
    t.lands(P0, "Island", 2);
    // The third instant: three counters, no transformation.
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(t.counters(amulet, "charge"), 3);
    assert_eq!(face(&t, amulet), FaceState::Front);
    // The fourth: P0 removes the four counters and transforms it.
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(amulet, "charge"), 0);
    assert_eq!(face(&t, amulet), FaceState::Back);
    assert_eq!(t.obj_now(amulet).chars.name, "Primal Wellspring");
}

#[test]
fn primal_wellsprings_copy_isnt_cast_and_is_controlled_by_you() {
    cr!("707.10", "601.2i", "106.6");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "If a copy is created, you control the copy. That copy is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger. The copy will then resolve like a normal spell, after players get a chance to cast spells and activate abilities."
    );
    supported("Primal Amulet // Primal Wellspring");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    let well = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    assert_eq!(t.obj_now(well).chars.name, "Primal Wellspring");
    t.battlefield(P0, "Young Pyromancer");
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(3),
    );
    assert!(tap_for_mana(&mut t, P0, well, "Add one mana"));
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    t.answer_yes(P0, false);
    t.settle();
    // The Wellspring's trigger and Young Pyromancer's are on the stack; once the
    // Wellspring's resolves, the copy waits on the stack above the Bolt.
    t.resolve();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    assert!(t.g.stack.contains(&bolt));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(elementals(&t, P0), 1);
}

#[test]
fn primal_wellsprings_copy_of_fling_deals_the_same_damage() {
    cr!("707.10", "707.2", "601.2f");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "For example, if you sacrifice a 3/3 creature to cast Fling, and you copy it, the copy of Fling will also deal 3 damage to its target."
    );
    supported("Primal Amulet // Primal Wellspring");
    supported("Fling");
    let mut t = TestGame::new(2);
    let well = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(3),
    );
    assert!(tap_for_mana(&mut t, P0, well, "Add one mana"));
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, fling).target(P1).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    // The copy dealt 3 damage too, and nothing else was sacrificed for it.
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}
