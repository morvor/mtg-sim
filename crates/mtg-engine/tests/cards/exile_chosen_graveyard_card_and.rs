//! "exile an artifact or creature card from your graveyard and put a cage counter on it"
//! (`oracle/patterns/exile_chosen_graveyard_card.rs`): within the sentence, "it" is the
//! exiled card, also in a triggered ability whose source would otherwise be "it"
//! (CR 400.7, 608.2c). (Mairsil, the Pretender says "from your hand or graveyard", which
//! isn't compiled yet; this is the graveyard-only form of the same sentence.)

use mtg_engine::card::{CardDef, Layout};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

const TEXT: &str = "When ~ enters, you may exile an artifact or creature card from your graveyard and put a cage counter on it.";

fn keeper() -> CardDef {
    let tl = TypeLine::parse("Creature — Vampire");
    let ctx = CompileContext {
        card_name: "Cage Keeper",
        full_name: "Cage Keeper",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("2"),
        toughness: Some("2"),
    };
    let compiled = oracle::compile(TEXT, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    CardDef::custom(Characteristics {
        name: SmolStr::new("Cage Keeper"),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        power: Some(2),
        toughness: Some(2),
        abilities: compiled.abilities,
        rules_text: Arc::from(TEXT),
        ..Default::default()
    })
}

#[test]
fn the_exiled_card_gets_the_counter_not_the_source() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let k = t.custom(P0, keeper(), Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.cast(P0, k).go();
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, "cage"), 1);
    let k = t.g.current(k);
    assert_eq!(t.zone(k), Zone::Battlefield);
    assert_eq!(t.counters(k, "cage"), 0);
}
