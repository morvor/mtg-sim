//! Shared helpers for the tests of rulings batch P088 (`r_p088_*.rs`): hatebears, the
//! creatures whose static abilities stop other players' (or everyone's) triggers, draws,
//! searches, counters, activated abilities and graveyard abilities. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Action, Answer};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p097_common::{enter_together, pool, tapped};
pub use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy};
pub use crate::r_s04_common::{activate_named, add_mana, on_stack, stack_items};
pub use crate::r_s06_common::activate_containing;
pub use crate::r_s25_common::cast_new;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether `p` could activate the activated ability of `source` whose text contains
/// `text` now (it's among `p`'s legal actions).
pub fn can_activate_containing(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    text: &str,
) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    let uids: Vec<u64> =
        t.g.obj(source)
            .chars
            .abilities
            .iter()
            .filter(|a| a.text.contains(text))
            .map(|a| a.uid)
            .collect();
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Activate { source: s, ability, .. } if *s == source && uids.contains(ability))
    })
}

/// Gives `p` `n` generic-usable colorless mana.
pub fn colorless(t: &mut TestGame, p: PlayerId, n: u32) {
    add_mana(t, p, ManaType::C, n);
}

/// Queues `p`'s answer "yes" to every yes/no question until cleared.
pub fn yes(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.answer(p, DecisionKind::YesNo, Answer::Bool(true));
    }
}

/// The special action ("pay {2} to ignore this effect", CR 116.2d) granted to `p` by a
/// static ability of `source`, if `p` could take it now.
pub fn static_action(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
) -> Option<mtg_engine::decision::SpecialAction> {
    use mtg_engine::decision::SpecialAction;
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).into_iter().find_map(|a| match a {
        Action::Special(s @ SpecialAction::Static { source: s2, .. }) if s2 == source => Some(s),
        _ => None,
    })
}

/// Takes a special action through the priority system.
pub fn take_special(t: &mut TestGame, p: PlayerId, sa: mtg_engine::decision::SpecialAction) {
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(sa));
    t.g.flush_events();
}
