//! Shared helpers for the tests of rulings batch P051 (`r_p051_*.rs`): "drain life"
//! effects (a player loses life or is dealt damage and you gain life), "draw matters" and
//! "draw hate" permanents, and "drain strength" effects (+X/+X to one creature and -X/-X
//! to another). (The helpers of batches S01–S30, P044–P057 are used too.)

#![allow(dead_code)]

use crate::r_p046_common::fill_pool;
use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// The life totals of the first `n` players.
pub fn lives(t: &TestGame, n: usize) -> Vec<i32> {
    (0..n).map(|i| t.life(PlayerId(i as u8))).collect()
}

/// `p` casts the real spell `name` paying from a pool of plenty of mana, with one target
/// answer per target slot, leaving it on the stack.
pub fn cast_pool(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    fill_pool(t, p, 10);
    let s = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, s).go()
}

/// Activates the activated ability of `source` whose text contains `needle` for `p` with
/// plenty of mana (one target answer per target slot); returns whether it was activated.
pub fn activate_text(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    targets: &[Entity],
) -> bool {
    fill_pool(t, p, 10);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    crate::r_s06_common::activate_containing(t, p, source, needle).is_ok()
}

/// Life gained by `p` this turn (the turn history).
pub fn gained_this_turn(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.life_gained.get(&p).copied().unwrap_or(0)
}

/// Cards `p` drew this turn (the turn history).
pub fn drawn_this_turn(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.cards_drawn.get(&p).copied().unwrap_or(0)
}
