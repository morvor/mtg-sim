//! Shared helpers for the tests of rulings batch P178 (`r_p178_*.rs`): "threaten"
//! effects (gain control until end of turn, untap, haste), Treasure Nabber and Frost
//! Titan, plus the cast, draw and enters triggers grouped with them. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p171_common::{cast_targeting, obj};
pub use crate::r_p177_common::{cast_resolve, concede, ffa};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, give_mana_for, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_attack, destroy};
pub use crate::r_s04_common::{stack_items, top_of_stack};
pub use crate::r_s06_common::{activate_containing, give_control, has_kw};
pub use crate::r_s14_common::triggers_from;
pub use crate::r_s24_common::{choose_creature_type, controller, enter_together, tapped};

/// Puts `n` filler cards into `p`'s graveyard (threshold).
pub fn fill_graveyard(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        let id =
            t.g.create_card_object(filler_card(), p, mtg_engine::object::Zone::Graveyard(p));
        t.g.players[p.idx()].graveyard.push(id);
    }
}

/// Whether the object (followed across zone changes) has haste now.
pub fn hasty(t: &TestGame, id: ObjectId) -> bool {
    has_kw(t, id, mtg_engine::keywords::KeywordKind::Haste)
}
