//! Rulings batch S02 — changeling (CR 702.73): "This object is every creature type."

use crate::r_s01_common::*;
use mtg_engine::ability::Filter;
use mtg_engine::eval::Ctx;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use smol_str::SmolStr;

/// Whether the object (followed across zones) matches "a [subtype]" as a filter would.
fn is_a(t: &TestGame, id: ObjectId, subtype: &str) -> bool {
    let id = t.g.current(id);
    t.g.matches(
        id,
        &Filter::Subtype(SmolStr::new(subtype)),
        &Ctx::new(None, P0),
    )
}

#[test]
fn a_changeling_card_is_a_mouse_frog_rabbit_lizard_and_brushwagg() {
    cr!("702.73a", "205.3m");
    ruling!(
        "Barkform Harvester",
        "A creature card with changeling is just as much a Mouse, a Frog, a Rabbit, a Lizard, and a Brushwagg as it is a Shapeshifter."
    );
    supported("Barkform Harvester");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Barkform Harvester");
    let harvester = t.battlefield(P0, "Barkform Harvester");
    for id in [in_hand, harvester] {
        for ty in ["Shapeshifter", "Mouse", "Frog", "Rabbit", "Lizard", "Brushwagg"] {
            assert!(is_a(&t, id, ty), "{ty}");
        }
    }
    assert_eq!(t.obj(in_hand).zone, Zone::Hand(P0));
    // Only creature types.
    assert!(!is_a(&t, harvester, "Forest"));
    assert!(!is_a(&t, harvester, "Aura"));
}

#[test]
fn a_creature_that_lost_all_creature_types_has_the_types_it_gains_later() {
    cr!("613.1d", "613.7", "205.3m");
    ruling!(
        "Amoeboid Changeling",
        "If a creature loses all creature types but then gains a new creature type later in the turn, it will be that new creature type."
    );
    supported("Nameless Inversion");
    supported("Amoeboid Changeling");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    // Amoeboid Changeling: "{T}: Target creature gains all creature types until end of
    // turn. {T}: Target creature loses all creature types until end of turn."
    let amoeboid = t.battlefield(P0, "Amoeboid Changeling");
    // Nameless Inversion: "Target creature gets +3/-3 and loses all creature types until
    // end of turn."
    t.lands(P0, "Swamp", 2);
    let inversion = t.hand(P0, "Nameless Inversion");
    t.cast(P0, inversion).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (9, 1));
    assert!(!is_a(&t, wurm, "Wurm"));
    assert!(!is_a(&t, wurm, "Elf"));
    // Later, it gains all creature types: it has them.
    t.activate(P0, amoeboid, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert!(is_a(&t, wurm, "Wurm"));
    assert!(is_a(&t, wurm, "Elf"));
    // Nameless Inversion's own changeling works in the graveyard.
    assert!(t.in_graveyard(P0, "Nameless Inversion"));
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Nameless Inversion")[0];
    assert!(is_a(&t, card, "Goblin"));
}

#[test]
fn a_creature_that_gained_all_creature_types_then_loses_them_has_none() {
    cr!("613.1d", "613.7");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let a = t.battlefield(P0, "Amoeboid Changeling");
    let b = t.battlefield(P0, "Amoeboid Changeling");
    t.activate(P0, a, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert!(is_a(&t, wurm, "Elf"));
    t.activate(P0, b, 1, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert!(!is_a(&t, wurm, "Elf"));
    assert!(!is_a(&t, wurm, "Wurm"));
    // The Amoeboid Changeling that lost nothing is still every creature type.
    assert!(is_a(&t, b, "Elf"));
}

#[test]
fn creatures_a_player_controls_lose_all_creature_types() {
    cr!("613.1d", "611.2c");
    supported("Ego Erasure");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let changeling = t.battlefield(P1, "Amoeboid Changeling");
    let mine = t.battlefield(P0, "Craw Wurm");
    // Ego Erasure: "Creatures target player controls get -2/-0 and lose all creature
    // types until end of turn."
    t.lands(P0, "Island", 3);
    let erasure = t.hand(P0, "Ego Erasure");
    t.cast(P0, erasure).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 4));
    assert!(!is_a(&t, wurm, "Wurm"));
    assert!(!is_a(&t, changeling, "Shapeshifter"));
    assert!(!is_a(&t, changeling, "Elf"));
    assert!(is_a(&t, mine, "Wurm"));
    // The creatures it affects were determined as it resolved: a creature P1 gets later
    // keeps its types and power.
    let later = t.enter(P1, "Craw Wurm");
    t.settle();
    assert!(is_a(&t, later, "Wurm"));
    assert_eq!(t.pt(later), (6, 4));
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(is_a(&t, wurm, "Wurm"));
    assert!(is_a(&t, changeling, "Elf"));
}
