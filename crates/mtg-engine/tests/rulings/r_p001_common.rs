//! Shared helpers for rulings batch P001 (`r_p001_*.rs`): Warhammer 40,000 cards, modal
//! Confluences and Seasons, prevention and life-gain replacement, cost reductions and
//! additional costs, graveyard-activated abilities and assorted damage spells.

#![allow(dead_code)]

pub use crate::r_p006_common::{obj, pl};
use mtg_engine::decision::{Action, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` could cast `card` normally now.
pub fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s02_common::can_cast(t, p, card, CastMethod::Normal)
}

/// `p` tries to cast `card` (no targets scripted); whether it was cast. A spell that is
/// cast must also have been offered as a legal action.
pub fn try_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let offered = castable(t, p, card);
    let cast = t.cast(p, card).try_go().is_ok();
    assert!(!cast || offered, "cast without being offered");
    cast
}

/// The players asked for priority since decision `from`.
pub fn priority_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    crate::r_p006_common::priority_since(t, from)
}

/// The mana value of the object (followed across zone changes) now.
pub fn mv(t: &TestGame, id: ObjectId) -> u32 {
    t.g.mana_value_of(t.g.current(id))
}

/// Whether `p` could activate the `index`th activated ability of `source` now.
pub fn can_activate_nth(t: &mut TestGame, p: PlayerId, source: ObjectId, index: usize) -> bool {
    use mtg_engine::ability::AbilityKind;
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    let Some(uid) = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(index)
        .map(|a| a.uid)
    else {
        return false;
    };
    t.g.legal_actions(p).iter().any(
        |a| matches!(a, Action::Activate { source: s, ability } if *s == source && *ability == uid),
    )
}

/// The options of the replacement-order choices asked of `p` since decision `from`.
pub fn replacement_choices_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseReplacement { options } if *q == p => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// Whether the permanent's controller is `p` (followed across zone changes).
pub fn controlled_by(t: &TestGame, id: ObjectId, p: PlayerId) -> bool {
    t.on_battlefield(id) && t.obj_now(id).controller == p
}

/// The number of untapped lands `p` controls.
pub fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s04_common::untapped_lands(t, p)
}

/// Puts `n` copies of the real card `name` into `p`'s graveyard.
pub fn graveyard_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}
