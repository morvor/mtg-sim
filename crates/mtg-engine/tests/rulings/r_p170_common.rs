//! Shared helpers for rulings batch P170 (`r_p170_*.rs`): noncreature-spell synergies
//! (abilities that trigger on casting noncreature spells, cost reductions, copies), party
//! counts and planeswalker synergies. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p012_common::cast_card;
pub use crate::r_p127_common::{move_to, tap};
pub use crate::r_s01_common::supported;
pub use crate::r_s04_common::{stack_items, top_of_stack};
pub use crate::r_s25_common::lands_for_cost;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Whether the topmost stack object is a triggered ability.
pub fn trigger_on_top(t: &TestGame) -> bool {
    t.g.stack.last().is_some_and(|id| {
        matches!(
            t.g.obj(*id).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Triggered { .. })
        )
    })
}

/// How a spell leaves the stack in response to the ability it caused to trigger.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Leaves {
    Stays,
    Countered,
    Bounced,
}

pub const ALL_LEAVES: [Leaves; 3] = [Leaves::Stays, Leaves::Countered, Leaves::Bounced];

/// The spell `spell` (controlled and owned by `p`) leaves the stack as `how` says: a
/// countered spell goes to its owner's graveyard (CR 701.6a), a bounced one to the hand.
pub fn leave(t: &mut TestGame, spell: ObjectId, p: PlayerId, how: Leaves) {
    match how {
        Leaves::Stays => {}
        Leaves::Countered => {
            assert!(t.g.counter(spell, None), "the spell is countered");
            assert_eq!(t.zone(spell), Zone::Graveyard(p), "countered");
        }
        Leaves::Bounced => {
            move_to(t, spell, Zone::Hand(p));
            assert_eq!(t.zone(spell), Zone::Hand(p), "bounced");
        }
    }
    assert_eq!(t.g.stack.contains(&spell), how == Leaves::Stays);
}

/// The tokens `p` controls whose name is `name`.
pub fn tokens_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// The tokens `p` controls with the subtype `sub`.
pub fn tokens_with_subtype(t: &TestGame, p: PlayerId, sub: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(sub))
        .map(|o| o.id)
        .collect()
}

/// Puts `n` +1/+1-style counters of `kind` on `id` as an effect would, and settles.
pub fn add_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.flush_events();
    t.settle();
}
