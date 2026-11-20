//! Shared helpers for the tests of item `rulings-residue-misc-2`, pass B
//! (`r_residue_misc_2b_*.rs`): the open rulings left in batches P156, P162, P163, P165,
//! P166, P168, P171, P172, P176, P177, P180, P185, P186, P189, P190, P193 and P194. Pass A's
//! helpers are reused.

#![allow(dead_code)]

pub use crate::r_residue_misc_2a_common::*;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The triggered abilities on the stack (bottom first) whose source is named `name`.
pub fn triggers_of(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|s| {
            matches!(&t.g.obj(*s).stack.as_ref().map(|si| &si.kind),
                Some(StackKind::Triggered { source, .. }) if t.g.obj(*source).chars.name == name)
        })
        .collect()
}

/// The controller of the stack object.
pub fn stack_controller(t: &TestGame, s: ObjectId) -> PlayerId {
    t.g.obj(s).controller
}

/// `p` casts Stifle (with an Island for it) at the stack object `what`, and it resolves
/// (only Stifle: the rest of the stack waits).
pub fn stifle(t: &mut TestGame, p: PlayerId, what: ObjectId) {
    t.lands(p, "Island", 1);
    let s = t.hand(p, "Stifle");
    t.cast(p, s).target(Entity::Object(what)).go();
    t.resolve();
}

/// Kills the creature (followed across zone changes) with a destroy effect, then settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.settle();
}

/// The number of permanents named `name` that `p` controls.
pub fn n_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj_now(*id).controller == p)
        .count()
}

/// Moves the object (followed across zone changes) to `zone` as an effect would, and
/// returns the new object.
pub fn mv(t: &mut TestGame, id: ObjectId, zone: mtg_engine::object::Zone) -> ObjectId {
    let id = t.g.current(id);
    let new =
        t.g.move_object(id, zone, mtg_engine::events::MoveCause::Effect, None)
            .expect("moved");
    t.g.flush_events();
    new
}

pub use crate::r_s02_common::create_token;

/// Puts the real card `name` onto the battlefield under `p`'s control tapped (it enters
/// tapped, as "put onto the battlefield tapped" would).
pub fn enter_tapped(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.hand(p, name);
    let id =
        t.g.move_object_ev(mtg_engine::replacement::MoveEv {
            obj: card,
            to: mtg_engine::object::Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                tapped: true,
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
    t.g.flush_events();
    id
}

/// A Commander game of `n` players.
pub fn commander_game(n: usize) -> TestGame {
    use mtg_engine::game::{GameConfig, Variant};
    TestGame::with_config(
        n,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Makes `id` one of its owner's commanders (CR 903.3).
pub fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.g.obj(id).owner;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
    t.g.dirty = true;
    t.g.recompute();
}

/// Taps the permanent (followed across zone changes) as an effect would; settles.
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}

pub use crate::r_p084_common::shuffles_since;
pub use crate::r_s01_common::{creatures, with_subtype};
