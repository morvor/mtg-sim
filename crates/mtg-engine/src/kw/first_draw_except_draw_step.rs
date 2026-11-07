//! `Condition::Custom` [`FIRST_DRAW_EXCEPT_DRAW_STEP`]: "The first time you would draw a
//! card each turn except the first card you draw during each of your draw steps" (Reed
//! Richards, Smartest Man): the draw being replaced is the player's first card draw this
//! turn not counting the first card they drew in each of their draw steps (CR 504.1,
//! 121.2). With [`crate::ability::PlayerFilter::FirstDrawInDrawStep`] excluded from the
//! event itself, this counts the draws proposed this turn (see `kw::first_draw_each_turn`)
//! less one for each draw step in which the player drew a card.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The `Condition::Custom` name.
pub const FIRST_DRAW_EXCEPT_DRAW_STEP: &str =
    "first draw event this turn except the first in each draw step";

pub struct FirstDrawExceptDrawStep;

impl KeywordRules for FirstDrawExceptDrawStep {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != FIRST_DRAW_EXCEPT_DRAW_STEP {
            return None;
        }
        let p = ctx
            .event
            .as_ref()
            .and_then(|e| e.player)
            .unwrap_or(ctx.controller);
        let proposed = g.history.draws_proposed.get(&p).copied().unwrap_or(0);
        let mut steps: Vec<usize> = g
            .history
            .draw_step_draws
            .iter()
            .filter(|(q, _, _)| *q == p)
            .map(|(_, s, _)| *s)
            .collect();
        steps.sort_unstable();
        steps.dedup();
        Some(proposed.saturating_sub(steps.len() as u32) == 1)
    }
}

inventory::submit! { KeywordRegistration(&FirstDrawExceptDrawStep) }
