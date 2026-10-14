//! Shared helpers for the tests of rulings batch P177 (`r_p177_*.rs`): stealing creatures
//! and permanents (temporary and lasting control changes, reanimating under your
//! control), stealing spells, and hand disruption ("target player reveals their hand").

#![allow(dead_code)]

use mtg_engine::decision::{Action, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p171_common::{cast_targeting, obj};
pub use crate::r_s01_common::{give_mana_for, supported};
pub use crate::r_s02_common::{can_activate, can_attack, destroy};
pub use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};
pub use crate::r_s24_common::{controller, tapped};

/// A free-for-all game with `n` players.
pub fn ffa(n: usize) -> TestGame {
    TestGame::with_config(n, GameConfig::free_for_all())
}

/// A player concedes, leaving the game (CR 104.3a, 800.4a).
pub fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
    t.settle();
}

/// Casts the real card `name` from `p`'s hand (paid with basic lands) with the given
/// targets, and resolves the stack. Returns the spell's id.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    let id = cast_targeting(t, p, name, targets);
    t.resolve_all();
    id
}

/// Sets the loyalty of a planeswalker.
pub fn set_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LOYALTY.into(), n);
}

/// The ids of the cards in `p`'s hand.
pub fn hand_ids(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).hand.clone()
}

/// Whether `p` was asked to choose among entities (cards) since decision `from`, with the
/// candidates of each such decision.
pub fn entity_choices_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}
