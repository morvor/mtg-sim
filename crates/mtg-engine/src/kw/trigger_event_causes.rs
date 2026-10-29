//! Why an object changed zones, for trigger conditions about the zone change
//! (`TriggerCond::Where` on a zone-change trigger; `EventInfo::cause`):
//!
//! * "Whenever a player mills a nonland card", "whenever one or more nonland cards are
//!   milled": a card moved from a library as the result of a mill action (CR 701.17a), to
//!   whatever public zone it went (CR 701.17c: a replacement effect may exile it instead).
//! * "Whenever one or more lands enter under an opponent's control without being played":
//!   the land didn't enter by being played (CR 305.1).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Condition::Custom`: the trigger event's object was milled.
pub const MILLED: &str = "zone change:milled";
/// `Condition::Custom`: the trigger event's permanent didn't enter by being played.
pub const NOT_PLAYED: &str = "zone change:not played";

pub struct TriggerEventCauses;

impl KeywordRules for TriggerEventCauses {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let cause = ctx.event.as_ref().and_then(|e| e.cause);
        match name {
            MILLED => Some(cause == Some(MoveCause::Mill)),
            NOT_PLAYED => Some(cause.is_some_and(|c| c != MoveCause::PlayLand)),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&TriggerEventCauses) }
