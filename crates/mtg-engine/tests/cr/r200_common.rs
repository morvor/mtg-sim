//! Shared helpers for the CR 200–213 tests (parts of a card).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::casting::CastOption;
use mtg_engine::object::{CastMethod, Characteristics, FaceState};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Queues the answer to `p`'s next "name a card" decision.
pub fn name_card(t: &mut TestGame, p: PlayerId, name: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(name.to_string()));
}

/// The card name chosen for `id` ("" if the answer named nothing).
pub fn chosen_name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id)
        .choices
        .card_name
        .as_deref()
        .unwrap_or("")
        .to_string()
}

/// A permanent with an "as this enters, choose a card name" ability (`chooser`, e.g.
/// Meddling Mage or Pithing Needle) enters under `p`'s control with `p` naming `name`.
/// Returns the permanent.
pub fn enter_naming(t: &mut TestGame, p: PlayerId, chooser: &str, name: &str) -> ObjectId {
    name_card(t, p, name);
    t.enter(p, chooser)
}

/// Whether `p` could begin to cast `card` using the given face or half right now.
pub fn can_cast_face(t: &mut TestGame, p: PlayerId, card: ObjectId, face: FaceState) -> bool {
    t.g.recompute();
    let mut opt = CastOption::normal(face);
    if let FaceState::Half(i) = face {
        opt.method = CastMethod::Half(i);
    }
    t.g.can_begin_cast(p, card, &opt)
}

/// Mana value of an object as the engine computes it (CR 202.3).
pub fn mv(t: &mut TestGame, id: ObjectId) -> u32 {
    t.g.recompute();
    t.g.mana_value_of(id)
}

/// A copy of a real card's definition with `names` interchangeable with its name
/// (CR 201.3).
pub fn with_interchangeable_names(name: &str, names: &[&str]) -> CardDef {
    let mut def = (*card(name)).clone();
    for f in def.faces.iter_mut() {
        f.chars.interchangeable_names = names.iter().map(|n| SmolStr::new(*n)).collect();
    }
    def
}

/// A custom card with the given characteristics (and no abilities).
pub fn plain_card(name: &str, type_line: &str, cost: &str, pt: Option<(i32, i32)>) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let m = mtg_engine::mana::ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(""),
        ..Default::default()
    })
}

/// Whether the object matches a filter, evaluated for `controller` with no source.
pub fn is(t: &TestGame, id: ObjectId, f: &Filter, controller: PlayerId) -> bool {
    t.g.matches(id, f, &mtg_engine::eval::Ctx::new(None, controller))
}
