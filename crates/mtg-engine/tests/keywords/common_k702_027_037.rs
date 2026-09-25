//! Shared helpers for the tests of CR 702.27–702.37 (buyback, shadow, cycling, echo,
//! horsemanship, fading, kicker, flashback, madness, fear, morph).

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Advances to `p`'s next upkeep (through the real turn structure) and puts the upkeep
/// triggers on the stack.
pub fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Number of untapped lands `p` controls.
pub fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.is(CardType::Land) && !o.tapped
        })
        .count()
}

/// Number of "Pay ...?" questions asked of `p` so far.
pub fn pay_questions(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")))
        .count()
}

/// The optional additional costs `p` was offered so far, by name.
pub fn optional_costs_offered(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.asked()
        .into_iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::OptionalCost { name, .. } => Some(name),
            _ => None,
        })
        .collect()
}

/// Whether casting `card` for `p` in the way `method` is currently a legal action.
pub fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, decision::Action::Cast { card: c, method: m } if *c == card && *m == method)
    })
}

/// The zone the card is in now (following it across zone changes).
pub fn zone_of(t: &TestGame, id: ObjectId) -> Zone {
    t.zone(id)
}

/// Activates the `nth` activated ability of `source` whose text is `text` (e.g. the
/// "Cycling" ability a cycling or typecycling keyword stands for).
pub fn activate_named(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    text: &str,
    nth: usize,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let source = t.g.current(source);
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| {
            matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)) && a.text == text
        })
        .nth(nth)
        .map(|a| a.uid)
        .expect("no such activated ability");
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, source, uid);
    t.g.flush_events();
    r
}

/// Activates the `nth` cycling ability of `card`.
pub fn cycle(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    nth: usize,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    activate_named(t, p, card, "Cycling", nth)
}

/// Number of creature tokens `p` controls.
pub fn tokens(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .count()
}
