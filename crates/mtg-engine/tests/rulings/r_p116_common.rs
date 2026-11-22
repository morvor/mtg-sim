//! Shared helpers for the tests of rulings batch P116 (`r_p116_*.rs`): stickers and
//! sticker kicker, Nicol Bolas, the Deceiver, "you can't lose the game", additional land
//! plays, sacrificing as a cost ("plunder"), poison counters, polymorph, "power doubler"
//! pumps, and power-matters abilities. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_s01_common::{custom_card, supported};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// The number of decisions asked so far.
pub fn n_asked(t: &TestGame) -> usize {
    t.asked().len()
}

/// Whether any player was asked for a priority decision since decision `from`.
pub fn priority_asked_since(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}

/// The `(min, max)` and candidates of the entity choices asked of `p` since `from`.
pub fn entity_choices_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<(u32, u32, usize)> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                min,
                max,
                candidates,
                ..
            } => Some((*min, *max, candidates.len())),
            _ => None,
        })
        .collect()
}

/// The number of lands `p` controls.
pub fn lands_controlled(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land))
        .count()
}

/// Plays as many Forests from `p`'s hand as allowed (up to `tries`), returning how many
/// were played.
pub fn play_forests(t: &mut TestGame, p: PlayerId, tries: usize) -> usize {
    let mut played = 0;
    for _ in 0..tries {
        let land = t.hand(p, "Forest");
        if t.play_land(p, land).is_ok() {
            played += 1;
        } else {
            // Leave the hand as it was.
            let land = t.g.current(land);
            t.g.move_object(
                land,
                mtg_engine::object::Zone::Exile,
                mtg_engine::events::MoveCause::Effect,
                None,
            );
        }
    }
    played
}

/// Sets a player's life total.
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

/// Puts real cards onto the battlefield under `p`'s control at the same time.
pub fn enter_together(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    use mtg_engine::events::MoveCause;
    use mtg_engine::object::Zone;
    use mtg_engine::replacement::{EtbInfo, MoveEv};
    let moves = names
        .iter()
        .map(|n| MoveEv {
            obj: t
                .g
                .create_card_object(mtg_engine::card::card(n), p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    t.g.move_objects(moves).into_iter().flatten().collect()
}

/// Casts the real card `name` for `p` (with lands for its cost) with the given targets
/// and resolves it (and everything else on the stack).
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    cast_new(t, p, name, targets);
    t.resolve_all();
}
