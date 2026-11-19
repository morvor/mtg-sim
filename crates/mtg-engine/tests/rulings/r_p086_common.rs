//! Shared helpers for the tests of rulings batch P086 (`r_p086_*.rs`): hate cards (cards
//! that punish or restrict noncreature spells, red, planeswalkers, nonbasic lands,
//! regeneration, and a set's mechanic or creature type). (The helpers of earlier batches
//! are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

pub use crate::r_p087_common::{activate, could_cast, is_tapped, obj, tap, untap};
pub use crate::r_s01_common::{attack_with, block_and_finish, supported};
pub use crate::r_s02_common::{create_token, destroy, target_candidates};
pub use crate::r_s03_common::{in_hand_with_mana, run_effect, to_blockers};
pub use crate::r_s04_common::{ability_targets, add_mana, next_upkeep, spell_targets};
pub use crate::r_s06_common::{attach_new, attached_to, damage, give_control};
pub use crate::r_s21_common::{castable, legal_blocks};
pub use crate::r_s25_common::{cast_new, lands_for_cost};
pub use crate::r_s29_common::{damage_marked, divide, put_counters};

/// Advances (everyone passing) to the precombat main phase of `p`'s next turn after the
/// current one.
pub fn next_main(t: &mut TestGame, p: PlayerId) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(20_000, |g| {
        g.turn.number > turn
            && g.turn.active == p
            && g.turn.step == Step::PrecombatMain
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(p)
    });
    assert!(ok, "did not reach {p}'s next main phase");
    t.settle();
}

/// Advances (everyone passing) to the upkeep of `p`'s next turn after the current one,
/// stopping before triggers are put on the stack.
pub fn next_upkeep_raw(t: &mut TestGame, p: PlayerId) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(20_000, |g| {
        g.turn.number > turn
            && g.turn.active == p
            && g.turn.step == Step::Upkeep
            && g.turn.stage == Stage::Priority
    });
    assert!(ok, "did not reach {p}'s next upkeep");
}

/// Whether `p` could cast a fresh copy of the card `name` now, with lands to pay for it.
pub fn can_cast_new(t: &mut TestGame, p: PlayerId, name: &str) -> bool {
    let c = in_hand_with_mana(t, p, name);
    castable(t, p, c)
}

/// The controller of the object (followed across zone changes).
pub fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}
