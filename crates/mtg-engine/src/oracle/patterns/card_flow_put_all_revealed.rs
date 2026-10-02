//! "Reveal the top three cards of your library. Put all creature cards revealed this way
//! into your hand and the rest into your graveyard." (Beast Hunt, Merfolk Wayfinder,
//! Goblin Ringleader, Garruk, Caller of Beasts): every revealed card of the kind is taken
//! — no choice — and the rest go where the text says (CR 701.20a). With fewer cards in
//! the library than that, all of them are revealed.

use super::card_flow_dig::{is_in_place, rest_destination, take_destination};
use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::FollowupPattern;

inventory::submit! {
    FollowupPattern { name: "card_flow: put all [cards] revealed this way into your hand and the rest ...", priority: 90, apply: put_all_revealed }
}

/// More than any library holds: "all" of the matching revealed cards.
const ALL: i32 = 9999;

fn put_all_revealed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("put all ") else {
        return false;
    };
    let Some((desc, r)) = r.split_once(" revealed this way ") else {
        return false;
    };
    // "cards of the chosen type" and the like refer to choices this form doesn't track.
    if desc.contains("chosen") {
        return false;
    }
    let Some(filter) = card_filter(desc, b) else {
        return false;
    };
    let Some((to, r)) = take_destination(r) else {
        return false;
    };
    if to.zone != ZoneKind::Hand {
        return false;
    }
    let Some(r) = r.strip_prefix(" and the rest ") else {
        return false;
    };
    let Some(rest) = rest_destination(r, false) else {
        return false;
    };
    match prev {
        Effect::Dig {
            who: PlayerRef::You,
            reveal: true,
            filter: f,
            take: take @ Value::Const(0),
            take_up_to,
            take_to,
            rest_to,
            ..
        } if is_in_place(rest_to) && rest.zone != ZoneKind::Hand => {
            *f = filter;
            *take = Value::c(ALL);
            *take_up_to = false;
            *take_to = to;
            *rest_to = rest;
            true
        }
        _ => false,
    }
}
