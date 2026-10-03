//! Shared helpers for rulings batch P037 (`r_p037_*.rs`): soft and reusable counterspells,
//! and abilities that create bigger bodies (tokens, animated lands and artifacts).

#![allow(dead_code)]

use mtg_engine::ability::AbilityKind;
use mtg_engine::eval::Ctx;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The legal choices for the first target of the activated ability of `source` whose text
/// contains `needle` (activated by its controller now).
pub fn ability_targets(t: &mut TestGame, source: ObjectId, needle: &str) -> Vec<Entity> {
    t.g.recompute();
    let s = t.g.current(source);
    let o = t.g.obj(s);
    let spec = o
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Activated(x) if a.text.contains(needle) => Some(x),
            _ => None,
        })
        .and_then(|a| a.body.targets.first().cloned())
        .expect("no targeted activated ability");
    let ctx = Ctx::new(Some(s), o.controller);
    t.g.legal_target_candidates(&spec, &ctx, s)
}

/// `p` casts Shock ("Shock deals 2 damage to any target.") at `target`, with a Mountain
/// to pay for it; returns the spell.
pub fn shock(t: &mut TestGame, p: PlayerId, target: impl Into<Entity>) -> ObjectId {
    t.lands(p, "Mountain", 1);
    let c = t.hand(p, "Shock");
    t.cast_with(p, c, &[target.into()]).expect("cast Shock")
}

/// Whether the object is a creature now (followed across zone changes).
pub fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// The subtypes of the object now.
pub fn subtypes(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Whether the permanent has an ability whose text contains `text`.
pub fn has_ability_text(t: &TestGame, id: ObjectId, text: &str) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains(text))
}

/// Tokens on the battlefield controlled by `p` with the given name.
pub fn tokens_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}
