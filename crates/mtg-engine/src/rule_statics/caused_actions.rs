//! Restrictions on what spells and abilities can make a player do (CR 701.9 discard,
//! CR 701.23 search), beside sacrifices ([`super::sacrifice_causes`]):
//!
//! * "Spells and abilities your opponents control can't cause you to discard cards."
//!   (Tamiyo, Collector of Tales): a discard an opponent's spell or ability would make you
//!   perform doesn't happen. Discarding to pay the costs of casting a spell or activating
//!   an ability isn't caused by a spell or ability (see `Game::discard`).
//! * "Spells and abilities your opponents control can't cause their controller to search
//!   their library." (Ashiok, Dream Render): an opponent instructed by their own spell or
//!   ability to search their library doesn't search it (anything else the spell or ability
//!   does, such as shuffling, still happens).
//!
//! Both are static abilities written as `StaticEffect::Custom` with these names.

use crate::eval::Ctx;
use crate::game::Game;
use crate::types::{ObjectId, PlayerId};

/// "Spells and abilities your opponents control can't cause you to discard cards."
pub const CANT_CAUSE_YOU_TO_DISCARD: &str =
    "spells and abilities your opponents control can't cause you to discard cards";
/// "Spells and abilities your opponents control can't cause their controller to search
/// their library."
pub const CANT_CAUSE_CONTROLLER_TO_SEARCH: &str =
    "spells and abilities your opponents control can't cause their controller to search their library";

fn active<'a>(g: &'a Game, name: &str) -> impl Iterator<Item = PlayerId> + 'a {
    let name = name.to_string();
    g.statics
        .customs
        .iter()
        .filter(move |(_, _, n)| n.as_str() == name)
        .map(|(_, c, _)| *c)
}

/// Whether `p` can't discard because of `source`'s effect: `by` is the player whose
/// spell or ability (from `source`) makes `p` discard, if it's caused by one.
pub fn discard_forbidden(g: &Game, p: PlayerId, by: Option<PlayerId>) -> bool {
    let Some(by) = by else {
        return false;
    };
    active(g, CANT_CAUSE_YOU_TO_DISCARD).any(|c| c == p && g.are_opponents(c, by))
}

/// Whether `searcher` can't search `owner`'s library as `ctx`'s spell or ability
/// instructs.
pub fn search_forbidden(g: &Game, searcher: PlayerId, owner: PlayerId, ctx: &Ctx) -> bool {
    if searcher != owner {
        return false;
    }
    let Some(cause) = super::sacrifice_causes::cause_of(ctx) else {
        return false;
    };
    let (Some(_), Some(by)) = (cause.obj, cause.by) else {
        return false;
    };
    by == searcher && active(g, CANT_CAUSE_CONTROLLER_TO_SEARCH).any(|c| g.are_opponents(c, by))
}

/// The player whose spell or ability `source` makes another player discard: its
/// controller, unless the discard pays the costs of casting a spell or activating an
/// ability.
pub fn discard_cause(g: &Game, source: Option<ObjectId>) -> Option<PlayerId> {
    match source {
        Some(s) if g.special.casting == 0 => Some(g.obj(s).controller),
        _ => None,
    }
}
