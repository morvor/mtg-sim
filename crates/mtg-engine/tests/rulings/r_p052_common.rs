//! Shared helpers for the tests of rulings batch P052 (`r_p052_*.rs`): "draw matters"
//! cards, drawbacks, and sacrificial artifacts ("eggs"). (The helpers of batches S01 and
//! P046 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Number of decisions asked so far (a mark for `asked_since`-style checks).
pub fn mark(t: &TestGame) -> usize {
    t.asked().len()
}

/// Draws `n` cards for `p` (one instruction) and settles.
pub fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.settle();
}

/// The players asked a decision matching `f` since decision `from`.
pub fn who_was_asked(t: &TestGame, from: usize, f: fn(&Decision) -> bool) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| f(d))
        .map(|(p, _)| *p)
        .collect()
}

/// Whether `id` (followed to its current object) has the keyword.
pub fn has_kw(t: &TestGame, id: ObjectId, k: mtg_engine::keywords::KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// Marks a permanent as having been on the battlefield since before this turn.
pub fn old(t: &mut TestGame, id: ObjectId) {
    let o = &mut t.g.objects[id.0 as usize];
    o.entered_turn = 0;
    o.summoning_sick = false;
}

/// Activates `source`'s `index`th activated ability for `p` with plenty of mana and
/// resolves the stack.
pub fn activate_resolve(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    index: usize,
    targets: &[Entity],
) {
    crate::r_p046_common::fill_pool(t, p, 10);
    t.activate(p, source, index, targets)
        .expect("activation failed");
    t.resolve_all();
}
