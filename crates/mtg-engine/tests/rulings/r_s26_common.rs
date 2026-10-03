//! Shared helpers for the tests of rulings batch S26 (`r_s26_*.rs`): rulings about copies
//! — token copies and what they copy, copies of objects that are copying something else,
//! copies of face-down permanents and of tokens, copies of spells (targets, modes, costs,
//! values of X, choices made on resolution), and the mana values of tokens and copies.
//! (The helpers of batches S01–S24 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::object::{ChosenMode, ObjKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Applies `mods` to the permanent until end of turn, as a resolving spell would (a
/// non-copy effect, CR 611.2), then settles.
pub fn modify_until_eot(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    let id = t.g.current(id);
    let mut ctx = mtg_engine::eval::Ctx::new(None, t.obj(id).controller);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Changes the permanent in every way a copy doesn't copy (CR 707.2): taps it, puts two
/// +1/+1 counters on it, and gives it +3/+3 and makes it green and blue until end of
/// turn.
pub fn dress_up(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, 2, None);
    modify_until_eot(
        t,
        id,
        vec![
            Modification::ModifyPT(Value::c(3), Value::c(3)),
            Modification::SetColors(ColorSet::single(Color::Green).union(ColorSet::single(Color::Blue))),
        ],
    );
}

/// Whether the permanent is untapped, has no counters, and nothing attached — as a new
/// token copy enters (CR 707.2).
pub fn fresh(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    !o.tapped
        && o.counters.values().all(|n| *n == 0)
        && t.g.attachments_of(Entity::Object(o.id)).is_empty()
}

/// The tokens `p` controls that entered after the battlefield snapshot `before`.
pub fn new_tokens(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| !before.contains(o) && t.obj(*o).is_token() && t.obj(*o).controller == p)
        .collect()
}

/// The modes and targets chosen for a spell or ability on the stack.
pub fn chosen_of(t: &TestGame, id: ObjectId) -> Vec<ChosenMode> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| si.chosen.clone())
        .unwrap_or_default()
}

/// The chosen mode indices of a stack object, in order.
pub fn modes_on_stack(t: &TestGame, id: ObjectId) -> Vec<usize> {
    chosen_of(t, id).iter().filter_map(|c| c.mode).collect()
}

/// Every target chosen for a stack object.
pub fn targets_on_stack(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    chosen_of(t, id)
        .iter()
        .flat_map(|m| m.targets.iter().flatten().copied())
        .collect()
}

/// The copies of spells on the stack, bottom first.
pub fn spell_copies(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy)
        .collect()
}

/// The mana value of the object now (CR 202.3).
pub fn mv(t: &mut TestGame, id: ObjectId) -> i64 {
    t.g.recompute();
    t.g.mana_value_of(t.g.current(id)) as i64
}
