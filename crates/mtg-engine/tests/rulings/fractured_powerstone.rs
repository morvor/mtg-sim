//! Rulings: Fractured Powerstone ("{T}: Roll the planar die. Activate only as a sorcery.").

use mtg_engine::card::Layout;
use mtg_engine::events::Event;
use mtg_engine::mana::ManaCost;
use mtg_engine::object::*;
use mtg_engine::planechase::ROLLED_PLANAR_DIE;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// An enchantment with the given oracle text, compiled as the card database would.
fn enchantment(name: &str, text: &str) -> CardDef {
    let tl = TypeLine::parse("Enchantment");
    let ctx = oracle::CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        mana_cost: ManaCost::parse("{1}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

#[test]
fn outside_a_planechase_game_rolling_the_planar_die_does_nothing() {
    ruling!(
        "Fractured Powerstone",
        "In non-Planechase games, Fractured Powerstone's second ability will have no effect."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let watcher = enchantment(
        "Die Watcher",
        "Whenever you roll the planar die, draw a card.",
    );
    assert!(watcher.faces[0]
        .chars
        .abilities
        .iter()
        .all(|a| !matches!(a.kind, mtg_engine::ability::AbilityKind::Unsupported(_))));
    t.custom(P0, watcher, Zone::Battlefield);
    t.library_top(P0, "Island");
    let stone = t.battlefield(P0, "Fractured Powerstone");
    t.activate(P0, stone, 1, &[]).unwrap();
    t.resolve_all();
    // The ability resolved (the Powerstone is tapped), but no planar die was rolled.
    assert!(t.obj(stone).tapped);
    assert!(t
        .turn_events
        .iter()
        .chain(t.events.iter())
        .all(|e| !matches!(e, Event::Custom { name, .. } if name == ROLLED_PLANAR_DIE)));
    assert_eq!(t.hand_size(P0), 0);
}
