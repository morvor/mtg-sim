//! "the number of cards put into their graveyard from anywhere this turn" (Fraying
//! Sanity): every card that was put into that player's graveyard this turn counts, even
//! if it has left the graveyard since, and even if the ability's source wasn't on the
//! battlefield at the time. Tokens aren't cards (CR 111.1). "Their" is the enchanted
//! player of the source (see `oracle/patterns/r404_graveyard_count_mill.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{PlayerRef, Sel};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;

/// `Value::Custom`: cards put into the enchanted player's graveyard this turn.
pub const CARDS_PUT_INTO_ENCHANTED_PLAYERS_GRAVEYARD: &str =
    "cards put into enchanted player's graveyard this turn";

pub struct CardsPutIntoGraveyard;

impl KeywordRules for CardsPutIntoGraveyard {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != CARDS_PUT_INTO_ENCHANTED_PLAYERS_GRAVEYARD {
            return None;
        }
        let Some(p) = g.eval_player(&PlayerRef::ControllerOf(Box::new(Sel::AttachedTo)), ctx)
        else {
            return Some(0);
        };
        Some(
            g.turn_events
                .iter()
                .filter(|e| match e {
                    Event::ZoneChange { new, to, .. } => {
                        *to == Zone::Graveyard(p) && !g.obj(*new).is_token()
                    }
                    _ => false,
                })
                .count() as i64,
        )
    }
}

inventory::submit! { KeywordRegistration(&CardsPutIntoGraveyard) }
