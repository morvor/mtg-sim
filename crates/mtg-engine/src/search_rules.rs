//! CR 701.23 Search.
//!
//! * Searching a hidden zone for cards with a stated quality, a player needn't find them
//!   (CR 701.23b); searching for a quantity of cards ("a card"), they must find that many
//!   if they can (CR 701.23d). A quality that's undefined matches no card (CR 701.23c).
//! * Found cards aren't revealed unless the effect says so (CR 701.23e).
//! * "If an opponent would search a library, that player searches the top four cards of
//!   that library instead." (Aven Mindcensor): the search is of that portion, and the
//!   rest of the effect still refers to the library (CR 701.23f).
//! * Searching a library several times before it's shuffled is a single search: it's
//!   reported once (CR 701.23h).
//! * Several players searching at once look at the cards at the same time, choose in
//!   APNAP order, and then the found cards move (CR 701.23i).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// `StaticEffect::Custom` prefix: "search portion:[N]:[who]" — a player searching a
/// library searches only its top N cards (who: "opponents", "you", or "each").
pub const SEARCH_PORTION: &str = "search portion:";

/// Searches of a library in progress: (searcher, library owner, the resolving spell or
/// ability). A search ends when that library is shuffled.
#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub open: Vec<(PlayerId, PlayerId, Option<ObjectId>)>,
}

/// How many cards from the top of `owner`'s library `searcher` searches, if an effect
/// limits the search to a portion of it (CR 701.23f).
pub fn portion(g: &Game, searcher: PlayerId, _owner: PlayerId) -> Option<usize> {
    let mut out: Option<usize> = None;
    for (_, ctl, name) in &g.statics.customs {
        let Some(spec) = name.strip_prefix(SEARCH_PORTION) else {
            continue;
        };
        let Some((k, who)) = spec.split_once(':') else {
            continue;
        };
        let Ok(k) = k.parse::<usize>() else {
            continue;
        };
        let applies = match who {
            "opponents" => g.are_opponents(*ctl, searcher),
            "you" => *ctl == searcher,
            "each" => true,
            _ => false,
        };
        if applies {
            out = Some(out.map_or(k, |o| o.min(k)));
        }
    }
    out
}

/// Whether a search is simply for a quantity of cards ("a card", "three cards"): the
/// filter states no quality (CR 701.23d).
pub fn quantity_only(f: &Filter) -> bool {
    match f {
        Filter::Any | Filter::Card | Filter::InZone(_) | Filter::OwnedBy(_) => true,
        Filter::And(v) => v.iter().all(quantity_only),
        _ => false,
    }
}

/// Records that `searcher` searches `owner`'s library for the resolving spell or ability
/// in `ctx`. Returns false if that's a further search of a library already being searched
/// by it (the same search, CR 701.23h).
pub fn begin(g: &mut Game, searcher: PlayerId, owner: PlayerId, ctx: &Ctx) -> bool {
    let key = (searcher, owner, ctx.stack_obj.or(ctx.source));
    // Only one spell or ability resolves at a time: searches for any other are over.
    g.searches.open.retain(|(_, _, s)| *s == key.2);
    if g.searches.open.contains(&key) {
        return false;
    }
    g.searches.open.push(key);
    true
}

/// A library was shuffled: searches of it are over.
pub fn library_shuffled(g: &mut Game, owner: PlayerId) {
    g.searches.open.retain(|(_, o, _)| *o != owner);
}
