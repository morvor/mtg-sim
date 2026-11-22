//! Shared helpers for the tests of rulings batch P182 (`r_p182_*.rs`): searching a library
//! for lands and other cards (CR 701.23) and the triggers, costs and modes around it.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p003_common::{activate, activate_resolve, cast, cast_resolve, mana, obj, pl};
pub use crate::r_p189_common::set_library;
pub use crate::r_s01_common::{attack_with, block_and_finish, supported};
pub use crate::r_s02_common::destroy;
pub use crate::r_s06_common::activate_containing;

/// One search choice a player was offered: the names of the candidates (sorted) and how
/// few and how many of them they had to and could find.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    pub names: Vec<String>,
    pub min: u32,
    pub max: u32,
}

/// The search choices `p` was offered since decision `from`, in order.
pub fn offers(t: &TestGame, p: PlayerId, from: usize) -> Vec<Offer> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                min,
                max,
                ..
            } if *q == p && prompt.starts_with("Search") => {
                let mut names: Vec<String> = candidates
                    .iter()
                    .filter_map(|e| e.object())
                    .map(|o| t.g.obj(o).chars.name.to_string())
                    .collect();
                names.sort();
                Some(Offer {
                    names,
                    min: *min,
                    max: *max,
                })
            }
            _ => None,
        })
        .collect()
}

/// The only search choice `p` was offered since decision `from`.
pub fn offer(t: &TestGame, p: PlayerId, from: usize) -> Offer {
    let o = offers(t, p, from);
    assert_eq!(o.len(), 1, "expected one search choice, got {o:?}");
    o[0].clone()
}

/// Sorted names, for comparing with [`Offer::names`].
pub fn names(v: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = v.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

/// How many times `p`'s library was shuffled this turn.
pub fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

/// How many times `p` searched a library this turn.
pub fn searches(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Searched { player, .. } if *player == p))
        .count()
}

/// The names of the permanents `p` controls (sorted).
pub fn permanents_of(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .permanents()
        .filter(|o| o.controller == p)
        .map(|o| o.chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// The names of the cards in `p`'s hand (sorted).
pub fn hand_of(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .player(p)
        .hand
        .iter()
        .map(|o| t.g.obj(*o).chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// Whether the object (followed across zone changes) is on the battlefield and tapped.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.on_battlefield(id) && t.g.obj(id).tapped
}

/// The number of lands `p` controls.
pub fn lands_of(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(mtg_engine::types::CardType::Land))
        .count()
}

/// Whether the object (followed across zone changes) is an attacking creature now.
pub fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.is_attacking(id)
}

