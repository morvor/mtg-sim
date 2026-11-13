//! Evaluates "if you had another creature enter the battlefield under your control last
//! turn" (`oracle/patterns/r603_entered_last_turn.rs`): a permanent that was a creature
//! as it entered the battlefield under the ability's controller's control during the
//! previous turn, other than the ability's source. What has happened to it since doesn't
//! matter, and a noncreature permanent that became a creature later doesn't count (Ephara,
//! God of the Polis's rulings).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::r603_entered_last_turn::CREATURE_ENTERED_LAST_TURN;
use crate::types::CardType;

pub struct EnteredLastTurn;

impl KeywordRules for EnteredLastTurn {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CREATURE_ENTERED_LAST_TURN {
            return None;
        }
        let source = ctx.source;
        Some(g.last_turn_history.permanents_entered.iter().any(|e| {
            e.controller == ctx.controller
                && Some(e.id) != source
                && match &e.as_entered {
                    Some((chars, _)) => chars.is(CardType::Creature),
                    None => g.obj(e.id).chars.is(CardType::Creature),
                }
        }))
    }
}

inventory::submit! { KeywordRegistration(&EnteredLastTurn) }
