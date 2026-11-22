//! "When a spell or ability an opponent controls causes you to discard this card, ..."
//! (Guerrilla Tactics, Psychic Purge): a triggered ability of the discarded card (CR
//! 701.9a) that triggers from wherever the card went (CR 113.6k), only when the discard
//! was an effect of a spell or ability controlled by an opponent of the player who
//! discarded it. Discarding the card to pay a cost, as a spell is cast or an ability
//! activated (CR 601.2h, 602.2b), isn't caused by a spell or ability's effect; nor is
//! discarding to the maximum hand size (CR 514.1). An optional discard ("you may discard
//! a card") an opponent's spell or ability offers still counts.
//!
//! The trigger is `TriggerCond::Custom` named [`DISCARDED_BY_OPPONENT`]; "that player" in
//! its effect is the opponent who controlled that spell or ability. The oracle phrase is
//! parsed in `oracle/patterns/a701_discard_caused_by_opponent.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::EventInfo;
use crate::types::*;

/// `TriggerCond::Custom` name of "when a spell or ability an opponent controls causes you
/// to discard ~".
pub const DISCARDED_BY_OPPONENT: &str = "a spell or ability an opponent controls causes you to discard this";

pub struct DiscardedByOpponent;

impl KeywordRules for DiscardedByOpponent {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != DISCARDED_BY_OPPONENT {
            return None;
        }
        Some(match ev {
            Event::Discarded {
                player,
                card,
                by: Some(by),
            } if *card == src && *player == ctl && g.are_opponents(*by, ctl) => {
                vec![EventInfo {
                    object: Some(src),
                    player: Some(*by),
                    ..Default::default()
                }]
            }
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&DiscardedByOpponent) }
