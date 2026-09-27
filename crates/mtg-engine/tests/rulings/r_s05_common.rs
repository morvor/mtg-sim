//! Shared helpers for the tests of rulings batch S05 (`r_s05_*.rs`): devoid (and the
//! Eldrazi Scions, ingest and processors of its sets), devour, disappear, discover,
//! disguise, Doctor's companion, domain, doubling, dredge, earthbend, echo, eerie, embalm,
//! emerge, eminence. (The helpers of batches S01, S02 and S04 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::Effect;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield under `p`'s control through a real zone
/// change (its enters abilities trigger), and puts the triggers on the stack.
pub fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// Tokens on the battlefield controlled by `p` with the given subtype.
pub fn tokens_with_subtype(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}

/// Moves `id` (following it across zone changes) to `zone` as an effect would, and
/// settles.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> Option<ObjectId> {
    let id = t.g.current(id);
    let new = t
        .g
        .move_object(id, zone, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
    new
}

/// Executes `effect` as if an ability of `source` controlled by `p` with the given targets
/// (one per slot) resolved, then settles.
pub fn run_from(
    t: &mut TestGame,
    p: PlayerId,
    source: Option<ObjectId>,
    effect: Effect,
    targets: &[Entity],
) {
    let mut ctx = mtg_engine::eval::Ctx::new(source, p);
    ctx.targets = targets.iter().map(|e| vec![*e]).collect();
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Whether the object (followed across zone changes) is colorless now.
pub fn colorless(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.colors.is_colorless()
}

/// The colors of the object (followed across zone changes) now.
pub fn colors(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}
