//! Helpers for the `basic_effects_*` card tests.

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Panics unless every ability of the card compiled.
pub fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// The candidates offered by the last target choice asked of `p`.
pub fn last_target_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no target choice was asked")
}

/// A noncreature card compiled from oracle text; panics if any of it isn't understood.
pub fn oracle_card(name: &str, type_line: &str, text: &str) -> card::CardDef {
    use mtg_engine::oracle::{self, CompileContext};
    let tl = types::TypeLine::parse(type_line);
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{name}: unsupported text {:?}",
        compiled.unsupported
    );
    card::CardDef::custom(mtg_engine::object::Characteristics {
        name: name.into(),
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        rules_text: std::sync::Arc::from(text),
        ..Default::default()
    })
}
