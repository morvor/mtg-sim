//! Shared helpers for the tests of rulings batch P181 (`r_p181_*.rs`): tutors and land
//! searches.

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p016_common::lots;
pub use crate::r_p123_common::pump;
pub use crate::r_p171_common::obj;
pub use crate::r_p177_common::cast_resolve;
pub use crate::r_p180_common::can_block;
pub use crate::r_p189_common::{bounce, set_library};
pub use crate::r_s01_common::block_and_finish;
pub use crate::r_s01_common::{attack_with, supported};
pub use crate::r_s02_common::create_token;
pub use crate::r_s02_common::destroy;
pub use crate::r_s06_common::activate_containing;
pub use crate::r_s11_common::triggered_from;

/// How many times `p` searched a library this turn (CR 701.23a).
pub fn searches(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Searched { player, .. } if *player == p))
        .count()
}

/// How many times `p`'s library was shuffled this turn.
pub fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

/// How many cards `p` drew this turn.
pub fn draws(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Drew { player, .. } if *player == p))
        .count()
}

/// The cards revealed this turn.
pub fn revealed(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == mtg_engine::reveal::REVEALED => *obj,
            _ => None,
        })
        .collect()
}

/// The permanents named `name` that `p` controls.
pub fn controlled(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.g.obj(*id).controller == p)
        .collect()
}

/// Whether the card (followed across zones) is in zone `z`.
pub fn is_in(t: &TestGame, id: ObjectId, z: Zone) -> bool {
    t.zone(t.g.current(id)) == z
}

/// The candidates of the last card choice `p` was asked since decision `from`.
pub fn last_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } if *q == p => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
}
