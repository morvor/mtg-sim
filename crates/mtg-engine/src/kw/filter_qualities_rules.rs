//! Object qualities, values and conditions of the filter and condition grammar
//! (`oracle/patterns/filter_qualities.rs`, `conditions_compare.rs`) implemented in code.
//! No keyword: only the registry-wide `custom_*` hooks.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::conditions_compare::PLAYERS_LOST_GAME;
use crate::types::*;

pub struct FilterQualities;

impl KeywordRules for FilterQualities {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<i64> {
        match name {
            // CR 800.4: players who have lost the game (or left it).
            PLAYERS_LOST_GAME => Some(
                g.player_ids()
                    .into_iter()
                    .filter(|p| {
                        let pl = g.player(*p);
                        pl.has_lost || pl.left_game
                    })
                    .count() as i64,
            ),
            _ => None,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let _ = (g, name, id, ctx);
        None
    }
}

inventory::submit! { KeywordRegistration(&FilterQualities) }
