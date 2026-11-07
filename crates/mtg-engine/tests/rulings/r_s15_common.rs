//! Shared helpers for the tests of rulings batch S15 (`r_s15_*.rs`): Role tokens, saddle,
//! scavenge, scry, secret council, shadow, skulk, sneak, solved, spectacle, spell mastery,
//! splice, split second, spree. (The helpers of batches S01–S11 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The Roles (Aura Role permanents) attached to `id` (followed across zone changes).
pub fn roles_on(t: &TestGame, id: ObjectId) -> Vec<ObjectId> {
    let id = t.g.current(id);
    t.g.permanents()
        .filter(|o| o.chars.has_subtype("Role") && o.attached_to == Some(Entity::Object(id)))
        .map(|o| o.id)
        .collect()
}

/// The names of the Roles attached to `id`.
pub fn role_names_on(t: &TestGame, id: ObjectId) -> Vec<String> {
    roles_on(t, id)
        .into_iter()
        .map(|r| t.obj(r).chars.name.to_string())
        .collect()
}

/// Number of tokens `p` created this game.
pub fn tokens_created_by(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.tokens_created.get(&p).copied().unwrap_or(0)
}

/// Number of "order these" decisions asked of `p` since decision `from`.
pub fn orders_asked_of(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::Order { .. }))
        .count()
}

/// The top `n` cards of `p`'s library, the top one first.
pub fn library_top_n(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .take(n)
        .copied()
        .collect()
}

/// The bottom `n` cards of `p`'s library, the bottom one first.
pub fn library_bottom_n(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p).library.iter().take(n).copied().collect()
}
