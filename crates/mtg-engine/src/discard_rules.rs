//! CR 701.9 Discard.
//!
//! * "If an effect causes you to discard a card" (Library of Leng) applies to discards
//!   caused by effects, not to discarding as a cost ([`DISCARDED_BY_EFFECT`]).
//! * A discarded card that an effect puts into a hidden zone instead of its owner's
//!   graveyard, without revealing it, has undefined characteristics (CR 701.9c): it's
//!   still discarded ("whenever a player discards a card" triggers), but nothing that
//!   asks about its characteristics ("whenever a player discards a creature card") sees
//!   one.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// `Filter::Custom` name: the card is being discarded because of an effect, not to pay a
/// cost (costs are paid while a spell is cast or an ability activated).
pub const DISCARDED_BY_EFFECT: &str = "discarded by an effect";

pub fn custom_filter(g: &Game, name: &str, _id: ObjectId, _ctx: &Ctx) -> Option<bool> {
    match name {
        DISCARDED_BY_EFFECT => Some(g.special.casting == 0),
        _ => None,
    }
}

/// Whether a filter asks nothing about the object's characteristics ("a card").
fn asks_nothing(f: &Filter) -> bool {
    match f {
        Filter::Any | Filter::Card => true,
        Filter::And(v) => v.iter().all(asks_nothing),
        _ => false,
    }
}

/// Whether the discarded card `card` (as it is now) matches `f`. A card put into a hidden
/// zone without being revealed has undefined characteristics (CR 701.9c).
pub fn discarded_card_matches(g: &Game, card: ObjectId, f: &Filter, ctx: &Ctx) -> bool {
    let hidden = matches!(g.obj(card).zone, Zone::Library(_) | Zone::Hand(_))
        && !crate::reveal::is_revealed(g, card);
    if hidden {
        return asks_nothing(f);
    }
    g.matches(card, f, ctx)
}
