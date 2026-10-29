//! Shared helpers for the tests of CR 702.168–702.177 (disguise, solved, plot, saddle,
//! spree, freerunning, gift, offspring, impending, exhaust).

#![allow(dead_code)]

use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;

/// Moves a permanent to exile and back onto the battlefield: it becomes a new object
/// (CR 400.7). Returns the new permanent.
pub fn flicker(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let id = t.g.current(id);
    let exiled =
        t.g.move_object(id, Zone::Exile, MoveCause::Effect, None)
            .expect("exiled");
    let back =
        t.g.move_object(exiled, Zone::Battlefield, MoveCause::Effect, None)
            .expect("returned");
    t.g.settle();
    back
}

/// The special actions `p` could take now (with priority).
pub fn special_actions(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

/// Takes a special action for `p`.
pub fn take_special(
    t: &mut TestGame,
    p: PlayerId,
    sa: SpecialAction,
) -> Result<(), mtg_engine::casting::Illegal> {
    t.g.turn.priority = Some(p);
    let r = t.g.perform_action(p, Action::Special(sa));
    t.g.flush_events();
    r
}

/// The ways `p` could cast `card` now.
pub fn cast_methods_now(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Cast { card: c, method } if c == card => Some(method),
            _ => None,
        })
        .collect()
}

/// Answers the next optional additional cost `p` is asked about.
pub fn pay_optional(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
}

/// The optional additional costs `p` has been asked about, by name.
pub fn optional_costs_asked(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::OptionalCost { name, .. } if q == p => Some(name),
            _ => None,
        })
        .collect()
}

/// Creature tokens `p` controls named `name`.
pub fn tokens_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.g.obj(*id).controller == p && t.g.obj(*id).is_token())
        .collect()
}

/// Tokens `p` controls with the subtype `subtype`.
pub fn tokens_of_subtype(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = t.g.obj(*id);
            o.controller == p && o.is_token() && o.chars.has_subtype(subtype)
        })
        .collect()
}
