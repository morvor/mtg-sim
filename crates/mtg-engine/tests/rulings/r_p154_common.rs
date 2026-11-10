//! Shared helpers for the tests of rulings batch P154 (`r_p154_*.rs`): abilities that
//! scale with a mana value (CR 202.3): X on and off the stack, lands and tokens, split
//! cards, commanders, cost reductions, triggers resolving before the spell they
//! triggered on, and spells whose targets became illegal. (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p003_common::{activate_resolve, cast, cast_resolve, damage, dmg, kill, mana, obj, pl, put};
pub use crate::r_p046_common::{cost_to_cast, others_asked_since, pool, tap_lands};
pub use crate::r_p048_common::controller;
pub use crate::r_p108_common::{end_step, resolved, two_headed_giant};
pub use crate::r_p117_common::{exile, gain_control, modify};
pub use crate::r_p151_common::act;
pub use crate::r_p152_common::{bounce, count_named, sacrifice, set_life};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, give_mana_for, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_cast, create_token, destroy, target_candidates};
pub use crate::r_s06_common::give_control;
pub use crate::r_s13_common::{commander, commander_game};
pub use crate::r_s25_common::{
    change_copy_targets, keep_copy_targets, lands_for_cost, cast_new, spell_copies, targets_of, x_of,
};

/// The mana value of the object (followed across zone changes) now (CR 202.3).
pub fn mana_value(t: &TestGame, id: ObjectId) -> u32 {
    let id = t.g.current(id);
    let o = t.g.obj(id);
    mtg_engine::mana_value::mana_value_with(&t.g, id, &o.chars)
}

/// Candidates of the entity choices asked of `p` since decision `from`.
pub fn entity_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// The names of the objects offered by the last entity choice asked of `p` since decision
/// `from` (sorted, without duplicates); empty if none was asked.
pub fn last_offered_names(t: &TestGame, p: PlayerId, from: usize) -> Vec<String> {
    let cands = entity_candidates(t, p, from);
    let mut names: Vec<String> = cands
        .last()
        .map(|c| {
            c.iter()
                .filter_map(|e| match e {
                    Entity::Object(o) => Some(t.g.obj(*o).chars.name.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.dedup();
    names
}

/// Whether any entity choice asked of `p` since decision `from` offered the object
/// (followed across zone changes).
pub fn was_offered(t: &TestGame, p: PlayerId, from: usize, id: ObjectId) -> bool {
    let now = t.g.current(id);
    entity_candidates(t, p, from)
        .iter()
        .any(|c| c.contains(&Entity::Object(id)) || c.contains(&Entity::Object(now)))
}

/// P0's commander `name`, on the battlefield under P1's control (a commander another
/// player controls, CR 903.3: still P0's commander).
pub fn stolen_commander(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P0, name);
    t.g.objects[c.0 as usize].is_commander = true;
    t.g.players[P0.idx()].commander_names.push(name.into());
    give_control(t, c, P1);
    assert_eq!(t.obj_now(c).controller, P1);
    c
}

/// P0's commander `name` on the battlefield under P0's control.
pub fn commander_on_battlefield(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P0, name);
    t.g.objects[c.0 as usize].is_commander = true;
    t.g.players[P0.idx()].commander_names.push(name.into());
    c
}

/// Endless One ({X}, mana value 0 off the stack) on `p`'s battlefield with `n` +1/+1
/// counters: a permanent whose X was not 0 as it was cast.
pub fn endless_one(t: &mut TestGame, p: PlayerId, n: u32) -> ObjectId {
    let e = t.battlefield(p, "Endless One");
    put(t, e, counters::PLUS1, n);
    e
}

/// The decisions asked of `p` since decision `from`.
pub fn asked_of(t: &TestGame, p: PlayerId, from: usize) -> Vec<Decision> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .map(|(_, d)| d.clone())
        .collect()
}
