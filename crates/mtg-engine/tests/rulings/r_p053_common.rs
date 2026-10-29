//! Shared helpers for the tests of rulings batch P053 (`r_p053_*.rs`): artifacts that
//! sacrifice themselves for value, energy generators, power/toughness boosts and setting,
//! and enchantment-cast / enchantment-enters engines. (The helpers of earlier batches are
//! used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p129_common::{give_energy, put_counters};
pub use crate::r_p130_common::{is_trigger, top};
pub use crate::r_s01_common::{attack_with, block_and_finish, supported};
pub use crate::r_s02_common::{can_activate, can_attack, destroy};
pub use crate::r_s06_common::{activate_containing, attach_new, attached_to, damage};
pub use crate::r_s10_common::{attacking, blocking};
pub use crate::r_s21_common::{blocks_now, legal_blocks};
pub use crate::r_s24_common::enter_together;
pub use crate::r_s28_common::energy;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Has `p` concede (CR 104.3a), then settles.
pub fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, mtg_engine::decision::Action::Concede);
    t.g.flush_events();
    t.settle();
}

/// Whether `p` could activate the activated ability of `source` whose text contains
/// `needle` now.
pub fn can_activate_text(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) -> bool {
    use mtg_engine::ability::AbilityKind;
    use mtg_engine::decision::Action;
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let s = t.g.current(source);
    let uid = t
        .g
        .obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such activated ability");
    let ok = t.g.legal_actions(p).iter().any(
        |a| matches!(a, Action::Activate { source, ability, .. } if *source == s && *ability == uid),
    );
    t.g.turn.priority = saved;
    ok
}

/// The non-priority decisions asked since decision `from`, for debugging.
pub fn asked_debug(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, mtg_engine::decision::Decision::Priority { .. }))
        .map(|(p, d)| format!("{p:?}: {d:?}"))
        .collect()
}
