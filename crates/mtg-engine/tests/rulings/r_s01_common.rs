//! Shared helpers for the tests of rulings batch S01 (`r_s01_*.rs`): adamant, adapt,
//! addendum, affinity, afflict, afterlife, aftermath, airbend, alliance, amass,
//! annihilator, ascend, assist, augment, awaken.

#![allow(dead_code)]

use mtg_engine::card::{card, CardDef, Layout};
use mtg_engine::decision::Decision;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Asserts that the real card's oracle text compiled completely, so the test exercises
/// the card as printed.
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// A custom card compiled from oracle text with the real oracle compiler (asserting it
/// compiled completely).
pub fn custom_card(
    name: &str,
    type_line: &str,
    cost: &str,
    pt: Option<(i32, i32)>,
    text: &str,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let p = pt.map(|x| x.0.to_string());
    let tt = pt.map(|x| x.1.to_string());
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: p.as_deref(),
        toughness: tt.as_deref(),
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{name}: unsupported text {:?}",
        compiled.unsupported
    );
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        mana_cost: mtg_engine::mana::ManaCost::parse(cost),
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// Puts real cards on top of `p`'s library, the first one on top.
pub fn stack_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    top_first
        .iter()
        .rev()
        .map(|n| t.library_top(p, n))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Lands for a spell's mana cost (basic lands for each colored symbol, Wastes for generic).
pub fn give_mana_for(t: &mut TestGame, p: PlayerId, name: &str) {
    let c = card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    let mut generic = 0usize;
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            n => {
                generic += n.parse::<usize>().unwrap_or(1);
                continue;
            }
        };
        t.lands(p, land, 1);
    }
    t.lands(p, "Wastes", generic);
}

/// Permanents on the battlefield controlled by `p` with the given subtype.
pub fn with_subtype(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}

/// Tokens on the battlefield controlled by `p`.
pub fn tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.id)
        .collect()
}

/// Creatures on the battlefield controlled by `p`.
pub fn creatures(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Creature))
        .map(|o| o.id)
        .collect()
}

/// Whether any lands `p` controls are tapped.
pub fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && o.tapped)
        .count()
}

/// The decisions asked since index `from` of the decision log.
pub fn asked_since(t: &TestGame, from: usize) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..].to_vec()
}

/// Puts the card `name` into `zone` for `p` (a real card).
pub fn place(t: &mut TestGame, p: PlayerId, name: &str, zone: Zone) -> ObjectId {
    t.custom(p, (*card(name)).clone(), zone)
}
