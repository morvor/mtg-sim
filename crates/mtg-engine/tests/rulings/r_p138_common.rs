//! Shared helpers for the tests of rulings batch P138 (`r_p138_*.rs`): "function:
//! removal-sacrifice" — edicts, sweepers that let players keep some permanents, and the
//! abilities that trigger on sacrifices and deaths.

#![allow(dead_code)]

pub use crate::r_p009_common::{kill, obj, pl};
pub use crate::r_p016_common::lots;
pub use crate::r_s01_common::{
    asked_since, attack_with, creatures, supported, triggers_on_stack, watch, Seen,
};
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts the real card `name` from hand paying from a full mana pool, with one answer
/// per target slot (each a list of targets).
pub fn try_cast(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    slots: &[&[Entity]],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    lots(t, p);
    lots(t, p);
    let c = t.hand(p, name);
    for s in slots {
        t.answer_targets(p, s);
    }
    t.g.turn.priority = Some(p);
    let r = t.g.cast_spell(p, c, CastMethod::Normal);
    t.g.flush_events();
    r
}

/// Like [`try_cast`], panicking if casting fails.
pub fn cast(t: &mut TestGame, p: PlayerId, name: &str, slots: &[&[Entity]]) -> ObjectId {
    try_cast(t, p, name, slots).unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// `p` sacrifices the permanent (as an instruction would have them) and the game settles,
/// putting any triggered abilities on the stack.
pub fn sac(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let id = t.g.current(id);
    t.g.sacrifice(id, p);
    t.g.flush_events();
    t.settle();
}

/// Moves an object to a zone (as an effect would), without settling.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> ObjectId {
    let id = t.g.current(id);
    let new = t
        .g
        .move_object(id, zone, MoveCause::Effect, None)
        .expect("moved");
    t.g.flush_events();
    new
}

/// The permanents `p` controls now.
pub fn permanents_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p)
        .map(|o| o.id)
        .collect()
}

/// Whether the object (followed through zone changes) is a permanent `p` controls.
pub fn controls(t: &TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.on_battlefield(id) && t.obj_now(id).controller == p
}

pub fn is_choose(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

/// The players asked to choose objects since decision `from`, in order.
pub fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    asked_since(t, from)
        .into_iter()
        .filter(|(_, d)| is_choose(d))
        .map(|(p, _)| p)
        .collect()
}

/// The candidates of the first object choice `p` was asked since `from`.
pub fn first_offer(t: &TestGame, p: PlayerId, from: usize) -> Vec<ObjectId> {
    asked_since(t, from)
        .into_iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => {
                Some(candidates.iter().filter_map(|e| e.object()).collect())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// Whether `p` was asked a yes/no question since `from`.
pub fn asked_yes_no(t: &TestGame, p: PlayerId, from: usize) -> bool {
    asked_since(t, from)
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
}

/// Watches the object choices of each player, recording the number of permanents on the
/// battlefield at each (to show nothing was sacrificed before everyone chose).
pub fn watch_all(t: &mut TestGame, players: &[PlayerId]) -> Vec<Seen<usize>> {
    players
        .iter()
        .map(|p| watch(t, *p, is_choose, |g| g.permanents().count()))
        .collect()
}

/// Deals lethal damage to all the given creatures at once, then settles: they die
/// simultaneously (CR 704.3) and their death triggers go on the stack.
pub fn kill_together(t: &mut TestGame, src: ObjectId, ids: &[ObjectId]) {
    let batch = ids
        .iter()
        .map(|id| (src, Entity::Object(t.g.current(*id)), 20))
        .collect();
    t.g.deal_damage_batch(batch, false);
    t.g.flush_events();
    t.settle();
}

/// Advances to `p`'s next upkeep (from any point in a turn) and settles.
pub fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        let n = t.g.players.len() as u8;
        let next = PlayerId((p.0 + 1) % n);
        t.advance_to(next, Step::PrecombatMain);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Advances to the end step of the current turn and settles.
pub fn to_end_step(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::End);
    t.settle();
}
