//! Shared helpers for the tests of rulings batch P123 (`r_p123_*.rs`): "can't block"
//! effects on one or many creatures, "can't have counters put on it", "whenever you gain
//! life" (Ajani's Pridemate and friends), exiled cards leaving exile, and protective
//! spells. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{Modification, Value};
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::two_headed_giant;
pub use crate::r_p120_common::{gain_life, give_minus1, give_plus1, plus1};
pub use crate::r_p122_common::{mana, obj, still_blocks_after};
pub use crate::r_s01_common::{attack_with, custom_card, supported, triggers_on_stack};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy};
pub use crate::r_s03_common::to_blockers;
pub use crate::r_s06_common::{activate_containing, attach_new};
pub use crate::r_s09_common::to_combat;
pub use crate::r_s10_common::blocking;
pub use crate::r_s21_common::{blocks_now, legal_blocks};
pub use crate::r_s25_common::lands_for_cost;
pub use crate::r_s26_common::modify_until_eot;
pub use crate::r_s28_common::cast_card;

/// Whether the creature (followed across zone changes) could block at all now (CR
/// 509.1a).
pub fn can_block(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.can_block_at_all(id)
}

/// The permanent gets +`p`/+`t` until end of turn.
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::Const(p), Value::Const(tough))],
    );
}

/// P0 casts the real card `name` (with lands for its cost) targeting `targets` (one slot)
/// and everything resolves.
pub fn cast_resolve_at(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    if !targets.is_empty() {
        t.answer_targets(p, targets);
    }
    cast_card(t, p, name);
    t.resolve_all();
}
