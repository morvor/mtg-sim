//! Shared helpers for the tests of rulings batch P040 (`r_p040_*.rs`): "whenever a creature
//! deals combat damage to a player, draw a card" abilities ("curiosity"), damage multipliers
//! and damage prevention. (The helpers of batches S01, S02, S04 and P039 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p039_common::{fight, mana_all, mark, o, old, pl, yes, yes_nos};

/// The index (among its activated abilities, for `TestGame::activate`) of the ability of
/// `id` whose text starts with `prefix`.
pub fn ability(t: &mut TestGame, id: ObjectId, prefix: &str) -> usize {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| {
            a.text
                .replace('−', "-")
                .starts_with(&prefix.replace('−', "-"))
        })
        .unwrap_or_else(|| panic!("no ability starting with {prefix:?}"))
}

/// `p` activates the loyalty (or other) ability of `pw` whose text starts with `prefix`
/// with the given targets (one per slot), and everything resolves.
pub fn loyalty(t: &mut TestGame, p: PlayerId, pw: ObjectId, prefix: &str, targets: &[Entity]) {
    let i = ability(t, pw, prefix);
    t.activate(p, pw, i, targets)
        .unwrap_or_else(|e| panic!("{prefix}: {e:?}"));
    t.resolve_all();
}

/// `p` casts the real card `name` from hand (lands for its mana cost are added) with the
/// given targets (one per slot) and everything resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, c).go();
    t.resolve_all();
}

/// Marked damage on the battlefield permanent `id`.
pub fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// The `lethal` lists of the combat damage assignment decisions asked since `from`.
pub fn lethal_offers(t: &TestGame, from: usize) -> Vec<Vec<u32>> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { lethal, .. } => Some(lethal),
            _ => None,
        })
        .collect()
}

/// Goes to the beginning of combat of `p`'s turn (without passing through earlier steps).
pub fn to_combat(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::BeginningOfCombat);
}

/// Whether `id`'s characteristics include card type `ty`.
pub fn is(t: &TestGame, id: ObjectId, ty: CardType) -> bool {
    t.obj_now(id).chars.card_types.contains(ty)
}

/// Whether `id` has subtype `st` now.
pub fn has_subtype(t: &TestGame, id: ObjectId, st: &str) -> bool {
    t.obj_now(id).chars.has_subtype(st)
}

/// Whether the log has a line containing `text`.
pub fn logged(t: &TestGame, text: &str) -> bool {
    t.g.log.iter().any(|l| l.text.contains(text))
}
