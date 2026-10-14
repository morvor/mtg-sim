//! Casting options for "the next spell you cast this turn" (CR 611.2f, 118.9, 601.3b;
//! `oracle/patterns/next_spell_options.rs`): an alternative cost offered to that spell
//! only, used up by the next matching spell however it's cast.

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The alternative cost an effect offers the spell (`kw/offered_costs.rs`).
const OFFERED: CastMethod = CastMethod::Alternative(mtg_engine::kw::offered_costs::OFFERED_ALT_COST);

#[test]
fn marshland_bloodcaster_next_spell_for_life() {
    cr!("118.9", "611.2f");
    assert_supported(&["Marshland Bloodcaster"]);
    let mut t = TestGame::new(2);
    let mb = t.battlefield(P0, "Marshland Bloodcaster");
    t.lands(P0, "Swamp", 2);
    let giant = t.hand(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    t.activate(P0, mb, 0, &[]).unwrap();
    t.resolve_all();
    t.cast(P0, giant).method(OFFERED).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Used up: the next spell can't be cast for life.
    assert!(t.cast(P0, bears).method(OFFERED).try_go().is_err());
}

#[test]
fn marshland_bloodcaster_is_used_up_by_a_spell_cast_normally() {
    cr!("611.2f");
    let mut t = TestGame::new(2);
    let mb = t.battlefield(P0, "Marshland Bloodcaster");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 1);
    let thopter = t.hand(P0, "Ornithopter");
    let giant = t.hand(P0, "Hill Giant");
    t.activate(P0, mb, 0, &[]).unwrap();
    t.resolve_all();
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert!(t.cast(P0, giant).method(OFFERED).try_go().is_err());
}

#[test]
fn world_war_hulk_next_red_or_green_creature_spell_is_free() {
    cr!("118.9", "611.2f", "714.2b");
    assert_supported(&["World War Hulk"]);
    let mut t = TestGame::new(2);
    t.enter(P0, "World War Hulk");
    t.settle();
    t.resolve_all();
    // A colorless creature spell isn't red or green: not free.
    let thopter = t.hand(P0, "Ornithopter");
    let giant = t.hand(P0, "Craw Wurm");
    let _ = thopter;
    t.cast(P0, giant).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    let mut t = TestGame::new(2);
    t.enter(P0, "World War Hulk");
    t.settle();
    t.resolve_all();
    let wall = t.hand(P0, "Steel Wall");
    assert!(t.cast(P0, wall).method(CastMethod::Free).try_go().is_err());
}

#[test]
fn savage_summoning_flash_uncounterable_and_a_counter() {
    cr!("601.3b", "611.2f", "614.1c", "400.7a");
    assert_supported(&["Savage Summoning"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let ss = t.hand(P0, "Savage Summoning");
    let bears = t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.cast(P0, ss).go();
    t.resolve_all();
    // During the upkeep: flash.
    let spell = t.cast(P0, bears).go();
    t.g.recompute();
    assert!(t
        .g
        .obj(spell)
        .chars
        .abilities
        .iter()
        .any(|a| a.text == "This spell can't be countered."));
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.pt(b), (3, 3));
    assert_eq!(t.zone(b), Zone::Battlefield);
}

#[test]
fn progenitors_icon_next_spell_of_the_chosen_type_has_flash() {
    cr!("601.3b", "611.2f");
    assert_supported(&["Progenitor's Icon"]);
    let mut t = TestGame::new(2);
    // Choose Bear.
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Bear")
        .expect("creature type");
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(i));
    let icon = t.enter(P0, "Progenitor's Icon");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.cast(P0, bears).try_go().is_err());
    t.activate(P0, icon, 1, &[]).unwrap();
    t.resolve_all();
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
