//! Game-history facts old (paper) cards ask about, compiled by
//! `oracle/patterns/paper_history.rs`:
//!
//! * [`STARTED_TURN_UNTAPPED`] (`Condition::Custom`): "if ~ started the turn untapped"
//!   (Rasputin Dreamweaver): the source was an untapped permanent as the turn began
//!   (`turn_start.rs`, recorded before the untap step, CR 502).
//! * [`REGENERATIONS_THIS_TURN`] (`Value::Custom`): "for each time it regenerated this
//!   turn" (Spiny Starfish): how many times the source regenerated this turn (CR 701.19a;
//!   its rulings: shields from any source count, and only shields that were used).
//! * [`DAMAGED_BY_SOURCE`] (`Condition::Custom` about the iterated player, and
//!   `Filter::Custom`): "each opponent and planeswalker it has dealt damage to this game"
//!   (The Fallen): the players and permanents the source, as the object it is now, has
//!   dealt damage to (its rulings: it forgets them once it leaves the battlefield, CR
//!   400.7, but not when it phases out). Damage events are kept for the whole game
//!   (`turn_start::kept_for_game`).
//! * [`NOT_SACRIFICED`] (`Filter::Custom`): "if it wasn't sacrificed" (Urza's Miter): the
//!   object a permanent became as it was put into a graveyard didn't get there by being
//!   sacrificed this turn (CR 701.21).
//! * [`NOT_FIRST_LAND_PLAYED`] (`Filter::Custom`): "if it wasn't the first land you played
//!   this turn" (Fastbond): a land played this turn after another land you played this
//!   turn (CR 305.2; lands put onto the battlefield aren't played, Fastbond's rulings).

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub const STARTED_TURN_UNTAPPED: &str = "paper history:~ started the turn untapped";
pub const REGENERATIONS_THIS_TURN: &str = "paper history:times ~ regenerated this turn";
pub const DAMAGED_BY_SOURCE: &str = "paper history:~ has dealt damage to it this game";
pub const NOT_SACRIFICED: &str = "paper history:it wasn't sacrificed";
pub const NOT_FIRST_LAND_PLAYED: &str = "paper history:it wasn't the first land you played this turn";

/// Whether the land `id` was played this turn after another land `p` played this turn.
fn not_first_land(g: &Game, id: ObjectId, p: PlayerId) -> bool {
    let plays: Vec<ObjectId> = g
        .turn_events
        .iter()
        .chain(g.events.iter())
        .filter_map(|e| match e {
            Event::LandPlayed { player, land } if *player == p => Some(*land),
            _ => None,
        })
        .collect();
    plays.iter().position(|l| *l == id).is_some_and(|i| i > 0)
}

/// Whether the source has dealt damage to `target` (this game, as the object it is now).
fn damaged_by_source(g: &Game, target: Entity, ctx: &Ctx) -> bool {
    let Some(src) = ctx.source else {
        return false;
    };
    g.game_events
        .iter()
        .chain(g.turn_events.iter())
        .chain(g.events.iter())
        .any(|e| matches!(e, Event::Damage { source, target: t, amount, .. } if *source == src && *t == target && *amount > 0))
}

struct PaperHistory;

impl super::KeywordRules for PaperHistory {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        match name {
            STARTED_TURN_UNTAPPED => Some(
                ctx.source
                    .is_some_and(|s| g.is_live(s) && g.turn_start.untapped.contains(&s)),
            ),
            DAMAGED_BY_SOURCE => Some(
                ctx.iter_player
                    .is_some_and(|p| damaged_by_source(g, Entity::Player(p), ctx)),
            ),
            _ => None,
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != REGENERATIONS_THIS_TURN {
            return None;
        }
        let Some(src) = ctx.source else {
            return Some(0);
        };
        Some(g.history.regenerated.iter().filter(|o| **o == src).count() as i64)
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        match name {
            DAMAGED_BY_SOURCE => Some(damaged_by_source(g, Entity::Object(id), ctx)),
            NOT_SACRIFICED => {
                let prev = g.obj(id).prev;
                Some(
                    !g.history
                        .sacrificed
                        .iter()
                        .any(|(_, s)| *s == id || Some(*s) == prev),
                )
            }
            NOT_FIRST_LAND_PLAYED => Some(not_first_land(g, id, ctx.controller)),
            _ => None,
        }
    }
}

inventory::submit! { super::KeywordRegistration(&PaperHistory) }
