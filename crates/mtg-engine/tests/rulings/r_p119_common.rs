//! Shared helpers for the tests of rulings batch P119 (`r_p119_*.rs`): "power matters"
//! abilities of a creature that look at its own power (last known information, values
//! fixed on resolution, damage divided as an ability is put on the stack) and "+1/+1
//! counters matter" abilities. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, resolved};
pub use crate::r_p116_common::{cast_resolve, n_asked, set_life};
pub use crate::r_p120_common::{gain_life, give_minus1, give_plus1, plus1, tokens_named};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, custom_card, stack_library, supported, triggers_on_stack,
};
pub use crate::r_s02_common::destroy;
pub use crate::r_s24_common::enter_together;
pub use crate::r_s25_common::{cast_new, creature_tokens, lands_for_cost};

/// A real card on the battlefield under `p`'s control with `n` +1/+1 counters on it.
pub fn with_counters(t: &mut TestGame, p: PlayerId, name: &str, n: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    if n > 0 {
        give_plus1(t, id, n);
    }
    id
}

/// P0 casts Giant Growth on `target`; only Giant Growth resolves.
pub fn giant_growth(t: &mut TestGame, target: ObjectId) {
    cast_new(t, P0, "Giant Growth", &[obj(target)]);
    t.resolve();
}

/// The number of objects on the stack.
pub fn stack(t: &TestGame) -> usize {
    t.g.stack.len()
}

/// Whether `id` (followed across zone changes) has the keyword.
pub fn has_kw(t: &mut TestGame, id: ObjectId, k: mtg_engine::keywords::KeywordKind) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.obj(id).has_keyword(k)
}

/// The total mana in `p`'s mana pool.
pub fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total() as usize
}
