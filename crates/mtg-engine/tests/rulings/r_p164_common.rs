//! Shared helpers for the tests of rulings batch P164 (`r_p164_*.rs`): activated-ability,
//! Adventure, Aura, basic-land, black and blocker synergies. (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p189_common::{bounce, set_library, triggers_of};
pub use crate::r_s01_common::{attack_with, creatures, supported, tokens};
pub use crate::r_s03_common::to_blockers;
pub use crate::r_s06_common::{give_control, has_kw};
pub use crate::r_s21_common::{blocks_now, legal_blocks};

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// The mana `p` pays to activate the `n`th activated ability of `src` (X announced as
/// `x`), with every cost change applied (CR 601.2f, 602.2b).
pub fn ability_mana(t: &mut TestGame, p: PlayerId, src: ObjectId, n: usize, x: u32) -> String {
    t.g.recompute();
    let a = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(n)
        .cloned()
        .expect("no such ability");
    let AbilityKind::Activated(act) = &a.kind else {
        unreachable!()
    };
    let act = act.clone();
    let c = t.g.ability_total_cost_with(p, src, &a, &act, None, x, None);
    c.mana.map(|m| m.to_string()).unwrap_or_default()
}

/// P1's `blocker` can block any number of creatures this turn (Valor Made Real).
pub fn valor_made_real(t: &mut TestGame, blocker: ObjectId) {
    supported("Valor Made Real");
    let p = t.g.obj(blocker).controller;
    t.lands(p, "Plains", 1);
    let v = t.hand(p, "Valor Made Real");
    t.cast(p, v).target(blocker).go();
    t.resolve_all();
}

/// The total life `p` gained this turn, one entry per event.
pub fn gains(t: &TestGame, p: PlayerId) -> Vec<u32> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::LifeGained { player, amount } if *player == p => Some(*amount),
            _ => None,
        })
        .collect()
}

/// The damage dealt this turn by `source` (followed by id): (recipient, amount).
pub fn damage_from(t: &TestGame, source: ObjectId) -> Vec<(Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source: s,
                target,
                amount,
                ..
            } if *s == source => Some((*target, *amount)),
            _ => None,
        })
        .collect()
}

/// The creatures attacking now.
pub fn attackers_now(t: &TestGame) -> Vec<ObjectId> {
    t.g.attackers()
}

/// The number of energy counters `p` has.
pub fn energy_of(t: &TestGame, p: PlayerId) -> u32 {
    crate::r_s28_common::energy(t, p)
}
