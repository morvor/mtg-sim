//! "Reveal the top card of your library. If it's a [kind of] card, put it [somewhere].
//! Otherwise, put it [somewhere else]." (Nissa, Sage Animist; Skyward Eye Prophets;
//! Zoologist; Garruk, Savage Herald).
//!
//! The first sentence is a single-card [`Effect::Dig`] (see `card_flow_dig`); the second
//! makes it take that card if it has the quality (the one revealed card is taken if it
//! qualifies, CR 608.2c), and the third says where the card goes if it doesn't. Without an
//! "Otherwise" sentence, a card that doesn't qualify stays on top of the library.
//!
//! Also "Look at the top card of your library. If it's a creature card, you may reveal it
//! and put it into your hand." (Domri Rade, Dryad Greenseeker): a looked-at card that may
//! be taken if it qualifies; otherwise it stays on top.

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::FollowupPattern;
use crate::oracle::phrases::end;

inventory::submit! {
    FollowupPattern { name: "card_flow: if it's a [card], put it ...", priority: 90, apply: if_its_a }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: otherwise, put it ...", priority: 90, apply: otherwise_put_it }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: if it's a [card], you may reveal it and put it into your hand", priority: 90, apply: may_reveal_if_its_a }
}

/// The revealed top card of your library, nothing decided yet.
fn single_reveal(prev: &mut Effect) -> Option<&mut Effect> {
    match prev {
        Effect::Dig {
            who: PlayerRef::You,
            n: Value::Const(1),
            reveal: true,
            take: Value::Const(0),
            rest_to,
            ..
        } if in_place(rest_to) => Some(prev),
        _ => None,
    }
}

fn in_place(d: &Destination) -> bool {
    d.zone == ZoneKind::Library && matches!(d.position, LibraryPosition::FromTop(0))
}

/// Where the card goes: "into your hand", "into your graveyard", "onto the battlefield",
/// "on the bottom of your library".
fn destination(s: &str) -> Option<Destination> {
    Some(match s {
        "into your hand" => Destination::zone(ZoneKind::Hand),
        "into your graveyard" => Destination::zone(ZoneKind::Graveyard),
        "onto the battlefield" => Destination::battlefield().under_your_control(),
        "onto the battlefield tapped" => Destination::battlefield().under_your_control().tapped(),
        "on the bottom of your library" => Destination::library_bottom(),
        _ => return None,
    })
}

/// "if it's a land card, put it onto the battlefield", "if it's a creature card, put it
/// into your hand".
fn if_its_a(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("if it's a ")
        .or_else(|| l.strip_prefix("if it's an "))
    else {
        return false;
    };
    let Some((desc, dest)) = r.split_once(", put it ") else {
        return false;
    };
    let (Some(filter), Some(to)) = (card_filter(desc, b), destination(dest)) else {
        return false;
    };
    // It stays in the library if it doesn't qualify, so it can't be put there.
    if to.zone == ZoneKind::Library {
        return false;
    }
    let Some(Effect::Dig {
        filter: f,
        take,
        take_up_to,
        take_to,
        ..
    }) = single_reveal(prev)
    else {
        return false;
    };
    *f = filter;
    *take = Value::c(1);
    *take_up_to = false;
    *take_to = to;
    true
}

/// "otherwise, put it into your hand", after "if it's a [card], put it ...".
fn otherwise_put_it(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("otherwise, put it ") else {
        return false;
    };
    let Some(rest) = destination(r) else {
        return false;
    };
    match prev {
        Effect::Dig {
            who: PlayerRef::You,
            n: Value::Const(1),
            reveal: true,
            take: Value::Const(1),
            take_up_to: false,
            take_to,
            rest_to,
            ..
        } if in_place(rest_to) && take_to.zone != rest.zone => {
            *rest_to = rest;
            true
        }
        _ => false,
    }
}

/// "if it's a creature card, you may reveal it and put it into your hand", after "look at
/// the top card of your library".
fn may_reveal_if_its_a(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("if it's a ")
        .or_else(|| l.strip_prefix("if it's an "))
    else {
        return false;
    };
    let Some(desc) = r.strip_suffix(", you may reveal it and put it into your hand") else {
        return false;
    };
    let Some(filter) = card_filter(desc, b) else {
        return false;
    };
    let Effect::Dig {
        who: PlayerRef::You,
        n: Value::Const(1),
        reveal: false,
        filter: f,
        take: take @ Value::Const(0),
        take_up_to,
        take_to,
        rest_to,
    } = prev
    else {
        return false;
    };
    if !in_place(rest_to) {
        return false;
    }
    *f = filter;
    *take = Value::c(1);
    *take_up_to = true;
    *take_to = Destination::zone(ZoneKind::Hand);
    true
}
