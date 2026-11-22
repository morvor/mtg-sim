//! "The first time you would draw a card each turn, ..." (Scion of Halaster's granted
//! ability): a draw replacement effect that applies only to the first draw event of that
//! player this turn. That draw counts even if another replacement effect replaced it (no
//! card was drawn), and a draw performed by the replacement's own instructions ("Then draw
//! a card.") isn't the first one (CR 614.1a, 121.2).
//! `Condition::Custom(FIRST_DRAW)`, evaluated with the draw event as the trigger event.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Condition::Custom` name: the draw event being replaced is the first one its player
/// would perform this turn.
pub const FIRST_DRAW: &str = "first draw event this turn";

pub struct FirstDrawEachTurn;

impl KeywordRules for FirstDrawEachTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != FIRST_DRAW {
            return None;
        }
        let p = ctx
            .event
            .as_ref()
            .and_then(|e| e.player)
            .unwrap_or(ctx.controller);
        Some(g.history.draws_proposed.get(&p).copied().unwrap_or(0) == 1)
    }
}

inventory::submit! { KeywordRegistration(&FirstDrawEachTurn) }
