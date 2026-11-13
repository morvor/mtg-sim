//! Shared helpers for rulings batch P061 (`r_p061_*.rs`): effects that make players draw
//! ("force draw"), creatures that attack each combat if able ("forced attacker"),
//! formidable, and free discard and sacrifice outlets. (The helpers of earlier batches are
//! used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

pub use crate::r_p059_common::{block, cast, cast_resolve, end_step_of, o, pl, put, run_to};
pub use crate::r_p060_common::{drawn_since, hands, libraries, library_of};
pub use crate::r_s01_common::{give_mana_for, stack_library, supported};
pub use crate::r_s06_common::{attach_new, give_control};
pub use crate::r_s09_common::{attacks_now, legal_attack};

/// A game of `n` players whose random choices come from `seed`.
pub fn seeded(n: usize, seed: u64) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            seed,
            ..Default::default()
        },
    )
}

/// From the current step of `p`'s turn (before combat), runs the game into `p`'s next
/// declare attackers step: the beginning of combat step happens (its triggers resolve,
/// everyone passing), then `p` declares `decl` (an illegal declaration is replaced by a
/// legal one, CR 508.1). Returns the attacks.
pub fn declare_from(
    t: &mut TestGame,
    p: PlayerId,
    decl: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, Entity)> {
    t.answer(p, DecisionKind::Attackers, Answer::Attackers(decl.to_vec()));
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers && g.turn.stage == Stage::Priority)
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
    attacks_now(t)
}

/// The activated ability of `id` whose text starts with `prefix`, as an index for
/// `TestGame::activate`.
pub fn ability(t: &mut TestGame, id: ObjectId, prefix: &str) -> usize {
    crate::r_p040_common::ability(t, id, prefix)
}

/// Whether the log has a line containing `text`.
pub fn logged(t: &TestGame, text: &str) -> bool {
    t.g.log.iter().any(|l| l.text.contains(text))
}
