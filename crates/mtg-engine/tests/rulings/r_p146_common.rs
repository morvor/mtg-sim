//! Shared helpers for the tests of rulings batch P146 (`r_p146_*.rs`): repeatable card
//! advantage and value engines (sacrifice outlets, "return a permanent you control",
//! rummaging, scry/surveil and Treasure engines). (The helpers of earlier batches are used
//! too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p057_common::{into_beginning_of_combat, into_upkeep, pump};
pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_p120_common::gain_life;
pub use crate::r_p160_common::{activate_resolve, cast_resolve, yes};
pub use crate::r_p223_common::{scries_since, scry_sizes};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, create_token, destroy};
pub use crate::r_s05_common::{move_to, run_from};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// The number of decisions asked so far.
pub fn n_asked(t: &TestGame) -> usize {
    t.asked().len()
}

/// `p` loses `n` life (as an effect would), then the game settles.
pub fn lose_life(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// The Treasure tokens `p` controls.
pub fn treasures(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Treasure"))
        .count()
}

/// Puts `n` filler cards into `p`'s hand.
pub fn fill_hand(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.hand(p, "Grizzly Bears")).collect()
}

/// `p` discards the card `card` (as an effect would), then the game settles.
pub fn discard(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    let card = t.g.current(card);
    t.g.discard(p, card, None);
    t.g.flush_events();
    t.settle();
}
