//! Cards in a graveyard qualified by how they got there this turn (cards3-cast-zones):
//! "cards in your graveyard that were put there from your library this turn" (Kagha,
//! Shadow Archdruid), "cards in graveyards that were put there from libraries this turn"
//! (Coram, the Undertaker), "... from anywhere other than the battlefield this turn"
//! (Banon, the Returners' Leader), "cards in your graveyard you've surveilled this turn"
//! (Eye of Duskmantle). A card that leaves the graveyard and returns is a new object
//! (CR 400.7), judged by its last trip there.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::ObjectId;

/// `Filter::Custom`: a card put into its current zone from a library this turn.
pub const PUT_THERE_FROM_LIBRARY: &str = "put there from a library this turn";
/// `Filter::Custom`: a card put into its current zone this turn from anywhere other than
/// the battlefield.
pub const PUT_THERE_NOT_FROM_BATTLEFIELD: &str =
    "put there from anywhere other than the battlefield this turn";
/// `Filter::Custom`: a card its owner put into their graveyard while surveilling this turn
/// (not a card they saw while surveilling and put there later by other means).
pub const SURVEILLED_THIS_TURN: &str = "surveilled this turn";

/// The zone the card was in before it was put into its current zone this turn, if it was
/// put there this turn.
fn came_from_this_turn(g: &Game, id: ObjectId) -> Option<Zone> {
    let o = g.try_obj(id)?;
    if o.entered_turn != g.turn.number {
        return None;
    }
    Some(g.try_obj(o.prev?)?.zone)
}

pub struct GraveyardHistory;

impl KeywordRules for GraveyardHistory {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        match name {
            PUT_THERE_FROM_LIBRARY => Some(matches!(
                came_from_this_turn(g, id),
                Some(Zone::Library(_))
            )),
            PUT_THERE_NOT_FROM_BATTLEFIELD => Some(
                came_from_this_turn(g, id).is_some_and(|z| z != Zone::Battlefield),
            ),
            SURVEILLED_THIS_TURN => Some(g.history.surveilled.contains(&id)),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&GraveyardHistory) }
