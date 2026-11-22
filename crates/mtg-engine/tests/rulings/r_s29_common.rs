//! Shared helpers for the tests of rulings batch S29 (`r_s29_*.rs`): rulings shared by
//! cards that mention counters — power/toughness layers and counters (CR 613.4), counters
//! on players and permanents (CR 122), stun counters, retrace, counterspells and spells
//! that can't be countered, ending the turn, and copies that keep a division (CR 707.10).
//! (The helpers of batches S01–S25 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the real card `name` into `p`'s hand with lands for its mana cost, then casts it
/// with the given targets (one per target slot), and resolves everything on the stack.
pub fn cast_and_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    crate::r_s25_common::cast_new(t, p, name, targets);
    t.resolve_all();
}

/// Puts `n` counters of `kind` on the object (with no source: its controller puts them)
/// and settles.
pub fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Queues `p`'s answer choosing modes by index.
pub fn choose_modes(t: &mut TestGame, p: PlayerId, modes: &[usize]) {
    t.answer(p, DecisionKind::Modes, Answer::Indices(modes.to_vec()));
}

/// Queues `p`'s answer dividing an amount among targets.
pub fn divide(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}

/// The damage marked on the object (followed across zone changes).
pub fn damage_marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// A scripted answer for some decisions (see `r_s03_common::respond`).
pub type Responder = fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>;

/// Answers a replacement-order choice (CR 616.1) with the option whose text contains
/// `needle`.
fn pick_replacement(d: &Decision, needle: &str) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.contains(needle))
            .map(Answer::Index),
        _ => None,
    }
}

/// Applies a "that many plus one" replacement effect first.
pub fn plus_one_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(d, "plus one")
}

/// Applies a "twice that many" replacement effect first.
pub fn twice_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(d, "twice")
}

/// The players asked to order replacement effects since decision `from`.
pub fn replacement_choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
        .map(|(p, _)| *p)
        .collect()
}
