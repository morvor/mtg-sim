//! CR 701.17 Mill.
//!
//! * A player mills as many cards as they can (CR 701.17b); a player can't choose to mill
//!   more cards than their library has, or pay such a cost (see `draw_rules::can_choose`
//!   and the mill cost part).
//! * The milled cards are found wherever they went from the library, if that's a public
//!   zone (CR 701.17c): "the milled card" can be a card exiled instead.
//! * Replacement effects can change how many cards are milled ("If an opponent would mill
//!   one or more cards, they mill twice that many cards instead."); information about "the
//!   milled card" then comes from each of them, summed for a value (CR 701.17d). They go
//!   through the replacement pipeline (`Game::replace_action`).

use crate::game::Game;
use crate::types::*;

/// `Condition::Custom`: "if two cards that share all their card types were milled this
/// way" (Demonic Covenant): the milled cards (`vars::IT`, CR 701.17c) are two with the same
/// set of card types (CR 205.2a; supertypes and subtypes don't count).
pub const TWO_MILLED_SHARE_ALL_TYPES: &str = "mill:two milled cards share all their card types";
/// `Condition::Custom`: "if two cards that share a card type were milled this way" (The
/// Tale of Tamiyo).
pub const TWO_MILLED_SHARE_A_TYPE: &str = "mill:two milled cards share a card type";

/// `Condition::Custom` conditions of this module.
pub fn custom_condition(g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<bool> {
    if name != TWO_MILLED_SHARE_ALL_TYPES && name != TWO_MILLED_SHARE_A_TYPE {
        return None;
    }
    let milled: Vec<ObjectId> = ctx
        .vars
        .get(&crate::ability::vars::IT)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default();
    let [a, b] = milled.as_slice() else {
        return Some(false);
    };
    let (ta, tb) = (g.obj(*a).chars.card_types, g.obj(*b).chars.card_types);
    Some(if name == TWO_MILLED_SHARE_ALL_TYPES {
        ta == tb
    } else {
        ta.intersects(tb)
    })
}
