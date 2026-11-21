//! Rulings batch P021 — a copy effect applies in layer 1, so effects that already applied
//! to a creature that becomes a copy keep applying to it, and power/toughness changes in
//! layer 7 apply on top of a copy's values no matter when they began (CR 613.1a, 613.4,
//! 707.2); an "{X}: base power and toughness X/X" ability isn't copiable (CR 707.2) and
//! does nothing to a noncreature (CR 208.3).

use crate::r_p021_common::*;
use crate::r_p023_common::{clone_of, enter_copying};
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::lands_for;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates Gigantoplasm's "{X}: This creature has base power and toughness X/X." with
/// X = `x` (paying with Wastes), and resolves it.
fn gigantoplasm_x(t: &mut TestGame, plasm: ObjectId, x: i64) {
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    activate_containing(t, P0, plasm, "base power and toughness X/X").expect("activate");
    t.resolve_all();
}

#[test]
fn permeating_mass_effects_already_applied_to_the_damaged_creature_continue() {
    cr!("613.1a", "613.4c", "707.2");
    ruling!(
        "Permeating Mass",
        "Effects that have already applied to the damaged creature will continue to apply to it. For example, if Giant Growth had given it +3/+3 earlier in the turn, it will be 4/6."
    );
    supported("Permeating Mass");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let mass = t.battlefield(P0, "Permeating Mass");
    let giant = t.battlefield(P1, "Hill Giant");
    giant_growth(&mut t, P1, giant);
    assert_eq!(t.pt(giant), (6, 6));
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[(mass, giant)]);
    t.resolve_all();
    let o = t.obj_now(giant);
    assert_eq!(o.chars.name, "Permeating Mass");
    assert_eq!(t.pt(giant), (4, 6));
    assert!(t.on_battlefield(giant));
}

#[test]
fn cytoshape_effects_already_applied_to_the_target_continue() {
    cr!("613.1a", "613.4c", "707.2");
    ruling!(
        "Cytoshape",
        "Effects that have already applied to the target creature will continue to apply to it. For example, if Giant Growth had given it +3/+3 earlier in the turn, then Cytoshape makes it a copy of Grizzly Bears, it will be a 5/5 Grizzly Bears."
    );
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    giant_growth(&mut t, P0, giant);
    lands_for(&mut t, P0, "{1}{G}{U}");
    let spell = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, spell).target(obj(giant)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn polymorphous_rush_effects_already_applied_to_the_targets_continue() {
    cr!("613.1a", "613.4c", "707.2");
    ruling!(
        "Polymorphous Rush",
        "Effects that have already applied to the target creatures will continue to apply to them."
    );
    supported("Polymorphous Rush");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    giant_growth(&mut t, P0, giant);
    lands_for(&mut t, P0, "{2}{U}{1}{U}");
    let spell = t.hand(P0, "Polymorphous Rush");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, spell).targets(&[obj(giant), obj(elves)]).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(giant), (5, 5));
    assert_eq!(t.obj_now(elves).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(elves), (2, 2));
}

#[test]
fn shapesharer_effects_already_applied_to_the_shapeshifter_continue() {
    cr!("613.1a", "613.4c", "707.2");
    ruling!(
        "Shapesharer",
        "Effects that have already applied to the targeted Shapeshifter will continue to apply to it."
    );
    supported("Shapesharer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sharer = t.battlefield(P0, "Shapesharer");
    giant_growth(&mut t, P0, sharer);
    assert_eq!(t.pt(sharer), (4, 4));
    lands_for(&mut t, P0, "{2}{U}");
    t.activate(P0, sharer, 0, &[obj(sharer), obj(bears)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.obj_now(sharer).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(sharer), (5, 5));
}

#[test]
fn gigantoplasm_power_changes_apply_no_matter_when_they_began() {
    cr!("613.4b", "613.4c", "707.9a");
    ruling!(
        "Gigantoplasm",
        "Effects that modify Gigantoplasm's power and/or toughness, such as the effect of Giant Growth or Glorious Anthem, will apply to Gigantoplasm no matter when they started applying."
    );
    supported("Gigantoplasm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Glorious Anthem");
    let plasm = enter_copying(&mut t, P0, "Gigantoplasm", bears);
    t.g.add_counters(obj(plasm), counters::PLUS1, 1, None);
    giant_growth(&mut t, P0, plasm);
    // 2/2 + 1 (Anthem) + 1 (counter) + 3 (Giant Growth).
    assert_eq!(t.pt(plasm), (7, 7));
    // Setting the base to 5/5 afterwards: everything still applies on top.
    gigantoplasm_x(&mut t, plasm, 5);
    assert_eq!(t.pt(plasm), (10, 10));
}

#[test]
fn gigantoplasms_x_ability_effect_isnt_copiable() {
    cr!("707.2", "613.4b");
    ruling!(
        "Gigantoplasm",
        "However, the effect of that ability isn't copiable."
    );
    supported("Gigantoplasm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let plasm = enter_copying(&mut t, P0, "Gigantoplasm", bears);
    gigantoplasm_x(&mut t, plasm, 4);
    assert_eq!(t.pt(plasm), (4, 4));
    let clone = clone_of(&mut t, P0, plasm);
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(clone), (2, 2));
    assert_eq!(abilities_with(&t, clone, "base power and toughness X/X"), 1);
}

#[test]
fn gigantoplasm_copying_an_animated_land_isnt_made_a_creature_by_its_ability() {
    cr!("707.2", "208.3", "613.4b");
    ruling!(
        "Gigantoplasm",
        "If Gigantoplasm isn't a creature (perhaps because it copied a land that had temporarily become a creature), you can still activate the ability"
    );
    supported("Gigantoplasm");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Mutavault");
    animate_land(&mut t, land);
    let plasm = enter_copying(&mut t, P0, "Gigantoplasm", land);
    assert_eq!(t.obj_now(plasm).chars.name, "Mutavault");
    assert!(t.obj_now(plasm).is(CardType::Land));
    assert!(!t.obj_now(plasm).is(CardType::Creature));
    gigantoplasm_x(&mut t, plasm, 3);
    assert!(!t.obj_now(plasm).is(CardType::Creature));
    assert!(t.on_battlefield(plasm));
}
