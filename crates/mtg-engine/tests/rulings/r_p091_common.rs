//! Shared helpers for the tests of rulings batch P091 (`r_p091_*.rs`): land count matters,
//! landfall, playing lands from other zones, "lands can't be played/enter".

#![allow(dead_code)]

use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Plays the land card `name` from `p`'s hand, then flushes events and puts triggers on
/// the stack.
pub fn play_from_hand(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.hand(p, name);
    t.play_land(p, id).expect("land play");
    t.g.flush_events();
    t.settle();
    t.g.current(id)
}

/// Puts the card `name` onto the battlefield under `p`'s control through a real zone
/// change (not played), then flushes events and puts triggers on the stack.
pub fn put_onto_battlefield(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// `p`'s legal actions with the card `card` (followed across zone changes).
pub fn actions_for(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<Action> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .into_iter()
        .filter(|a| match a {
            Action::PlayLand { card: c } | Action::Cast { card: c, .. } => *c == card,
            Action::Activate { source, .. } => *source == card,
            _ => false,
        })
        .collect()
}

/// Whether `p` may play the land `card` now.
pub fn may_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    actions_for(t, p, card)
        .iter()
        .any(|a| matches!(a, Action::PlayLand { .. }))
}

/// Whether `p` may cast `card` now.
pub fn may_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    actions_for(t, p, card)
        .iter()
        .any(|a| matches!(a, Action::Cast { .. }))
}

/// Number of triggered abilities on the stack whose source is `source`.
pub fn triggers_of(t: &TestGame, source: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(si.kind, mtg_engine::object::StackKind::Triggered { source: s, .. }
                    if s == source)
            })
        })
        .count()
}
