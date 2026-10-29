//! Shared helpers for the tests of rulings batch P151 (`r_p151_*.rs`): permanents that
//! sacrifice themselves (as a cost, or as part of a triggered ability's effect), and
//! sacrifice outlets (sacrificing any permanents, or lands). (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters};
pub use crate::r_p117_common::{exile, gain_control, modify};
pub use crate::r_p119_common::{counter_spell, wrath};
pub use crate::r_s01_common::{attack_with, block_and_finish, supported, tokens};
pub use crate::r_s02_common::{can_activate, destroy};
pub use crate::r_s25_common::cast_new;

/// The number of triggered or activated abilities on the stack.
pub fn abilities_on_stack(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                !matches!(si.kind, mtg_engine::object::StackKind::Spell { .. })
            })
        })
        .count()
}

/// Removes `n` counters of `kind` from the object and settles.
pub fn remove_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.remove_counters(Entity::Object(id), kind, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Returns a card in a graveyard or exile to the battlefield under `p`'s control.
pub fn reanimate(t: &mut TestGame, p: PlayerId, id: ObjectId) -> ObjectId {
    let id = t.g.current(id);
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .expect("returned to the battlefield");
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    new
}

/// `src` deals `n` noncombat damage to `to`, then the game settles.
pub fn damage(t: &mut TestGame, src: ObjectId, to: Entity, n: u32) {
    t.g.deal_damage(src, to, n, false);
    t.g.flush_events();
    t.settle();
}

/// Permanents `p` controls.
pub fn permanents_of(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents().filter(|o| o.controller == p).count()
}

/// Advances to P0's next upkeep (through P1's turn), its triggers on the stack.
pub fn next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.settle();
}

/// The index (among its activated abilities, as `TestGame::activate` counts them) of the
/// activated ability of `src` whose text contains `text`.
pub fn ability_index(t: &mut TestGame, src: ObjectId, text: &str) -> usize {
    t.g.recompute();
    t.g.obj(t.g.current(src))
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.contains(text))
        .unwrap_or_else(|| panic!("no activated ability with {text:?}"))
}

/// `p` activates the ability of `src` whose text contains `text`.
pub fn act(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    text: &str,
    targets: &[Entity],
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    let i = ability_index(t, src, text);
    t.activate(p, src, i, targets)
}

/// `p` gains control of the permanent indefinitely, then the game settles.
pub fn steal_for_good(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}
