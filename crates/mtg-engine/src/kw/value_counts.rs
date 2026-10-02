//! Counts of players used by the value grammar (`oracle/patterns/value_grammar.rs`):
//! "the number of opponents being attacked", "for each opponent you're attacking" — the
//! opponents at least one attacking creature is attacking in the current combat (a
//! creature attacking a planeswalker or battle doesn't attack its controller or
//! protector, CR 506.3d).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Value::Custom` name: the number of the controller's opponents being attacked.
pub const OPPONENTS_BEING_ATTACKED: &str = "opponents being attacked";

/// `Value::Custom` name: the number of opponents the controller attacked with one or
/// more creatures this turn (CR 508.6).
pub const OPPONENTS_ATTACKED_THIS_TURN: &str = "opponents you attacked this turn";

/// `Value::Custom` name: the number of players who have lost the game (CR 104.3).
pub const PLAYERS_WHO_LOST: &str = "players who have lost the game";

pub struct ValueCounts;

impl KeywordRules for ValueCounts {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        match name {
            OPPONENTS_BEING_ATTACKED => Some(
                super::players_being_attacked::players_being_attacked(g)
                    .into_iter()
                    .filter(|p| g.are_opponents(ctx.controller, *p))
                    .count() as i64,
            ),
            OPPONENTS_ATTACKED_THIS_TURN => Some(
                g.players_in_game()
                    .into_iter()
                    .filter(|p| {
                        g.are_opponents(ctx.controller, *p)
                            && crate::combat::player_has_attacked(g, ctx.controller, *p)
                    })
                    .count() as i64,
            ),
            PLAYERS_WHO_LOST => Some(g.players.iter().filter(|p| p.has_lost).count() as i64),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&ValueCounts) }
