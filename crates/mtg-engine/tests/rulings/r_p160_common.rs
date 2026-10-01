//! Shared helpers for the tests of rulings batch P160 (`r_p160_*.rs`): "shade pump"
//! activated abilities (+X/+X effects whose X and affected set are locked in as they
//! resolve), "shakedown", and "shapechange" effects that set base power and toughness
//! (CR 613.4b) or copy objects. (The helpers of batches S01–S30 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_s01_common::supported;
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Casts the real card `name` (with lands for its cost) with the given targets and
/// resolves the stack.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    cast_new(t, p, name, targets);
    t.resolve_all();
}

/// Activates the `index`th activated ability of `source` and resolves the stack.
pub fn activate_resolve(t: &mut TestGame, p: PlayerId, source: ObjectId, index: usize, targets: &[Entity]) {
    t.activate(p, source, index, targets)
        .unwrap_or_else(|e| panic!("activation failed: {e:?}"));
    t.resolve_all();
}

/// Puts `n` +1/+1 counters on the object.
pub fn plus_counters(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Queues a "yes" for `p`'s next yes/no decision.
pub fn yes(t: &mut TestGame, p: PlayerId) {
    t.answer(p, DecisionKind::YesNo, Answer::Bool(true));
}

/// The creature tokens `p` controls.
pub fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.id)
        .collect()
}
