//! Shared helpers for the tests of rulings batch P044 (`r_p044_*.rs`): death triggers
//! (CR 603.6c, 603.10a), delayed triggers (CR 603.7), and the abilities of old legendary
//! creatures. (The helpers of batch S01 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts the real spell `name` (with the mana for it), with the given targets, and
/// everything resolves.
pub fn cast_spell(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let s = cast_only(t, p, name, targets);
    t.resolve_all();
    s
}

/// `p` casts Wrath of God (destroy all creatures); everything resolves.
pub fn wrath(t: &mut TestGame, p: PlayerId) {
    cast_spell(t, p, "Wrath of God", &[]);
}

/// `p` casts Planar Cleansing (destroy all nonland permanents); everything resolves.
pub fn cleansing(t: &mut TestGame, p: PlayerId) {
    cast_spell(t, p, "Planar Cleansing", &[]);
}

/// Deals damage to several creatures at once (one simultaneous event) from a source P1
/// controls, then checks state-based actions and puts triggers on the stack.
pub fn damage_at_once(t: &mut TestGame, hits: &[(Entity, u32)]) {
    let src = t.battlefield(P1, "Mountain");
    t.g.deal_damage_batch(hits.iter().map(|(e, n)| (src, *e, *n)).collect(), false);
    t.settle();
}

/// The objects controlled by `p` (on the battlefield) with this name.
pub fn named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
pub fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

/// `p` loses the game at the same time as the creature `name` it controls dies: P1 casts
/// Earthquake for `x` with `p` at `x` life (three players, so the game goes on).
pub fn dies_with_its_controller(t: &mut TestGame, p: PlayerId, x: i64) {
    supported("Earthquake");
    t.g.players[p.idx()].life = x as i32;
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    give_mana_for(t, P1, "Earthquake");
    t.lands(P1, "Wastes", x as usize - 1);
    let s = t.hand(P1, "Earthquake");
    t.cast(P1, s).x(x).go();
    t.resolve_all();
}

/// `p` casts the real spell `name` (with the mana for it) with the given targets, leaving
/// it on the stack.
pub fn cast_only(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    give_mana_for(t, p, name);
    let s = t.hand(p, name);
    t.g.turn.priority = Some(p);
    // (No queued target answer for a spell without targets: it would be left over.)
    let b = t.cast(p, s);
    let b = if targets.is_empty() {
        b
    } else {
        b.targets(targets)
    };
    b.go()
}

/// Destroys a permanent outside of any spell (as a response would) and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.settle();
}

/// Number of `kind` counters player `p` has.
pub fn player_counters(t: &TestGame, p: PlayerId, kind: &str) -> u32 {
    t.g.player(p).counters.get(kind).copied().unwrap_or(0)
}
