//! Shared helpers for the tests of rulings batch P162 (`r_p162_*.rs`): putting lands
//! onto the battlefield, self-including death and enter triggers, damage-reflection
//! triggers, state triggers, untap limits, spell counts, strive and "up to N targets"
//! spells, sunburst and converge, surveil, and swap removal. (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Lands `p` has played this turn.
pub fn lands_played(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).lands_played_this_turn
}

/// The candidates of the "choose entities" decisions asked since decision `from`.
pub fn chosen_from(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Casts the real card `name` from `p`'s hand (adding lands for its mana cost) with the
/// given targets (one slot each) and resolves the stack.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    crate::r_s01_common::give_mana_for(t, p, name);
    let c = t.hand(p, name);
    let mut b = t.cast(p, c);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    t.resolve_all();
    c
}
