//! "Whenever an opponent draws a card except the first one they draw in each of their
//! draw steps" (Orcish Bowmasters, Leela, Sevateem Warrior; Xyris, the Writhing Storm):
//! the card a player draws in their draw step's turn-based action (CR 504.1) — or, if
//! that draw was replaced, the first card they do draw in that step — doesn't trigger
//! them. Which draws those were is recorded in `TurnHistory::draw_step_draws` (see
//! `draw_rules`); the replacement effects that make the same exception use
//! `PlayerFilter::FirstDrawInDrawStep`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Condition::Custom`, for a trigger condition about a card being drawn
/// (`TriggerCond::Where`): the drawn card wasn't the first one its drawer drew in one of
/// their draw steps.
pub const NOT_FIRST_DRAW_IN_DRAW_STEP: &str =
    "draw step:not the first card drawn in a draw step";

pub struct DrawStepDraws;

impl KeywordRules for DrawStepDraws {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != NOT_FIRST_DRAW_IN_DRAW_STEP {
            return None;
        }
        let card = ctx.event.as_ref().and_then(|e| e.object);
        Some(card.is_some_and(|c| !crate::draw_rules::was_first_draw_in_draw_step(g, c)))
    }
}

inventory::submit! { KeywordRegistration(&DrawStepDraws) }
