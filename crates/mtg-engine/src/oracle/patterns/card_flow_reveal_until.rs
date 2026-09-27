//! Revealing cards from the top of a library until a card with some quality is revealed
//! (CR 701.20): "Reveal cards from the top of your library until you reveal a creature
//! card." followed by where "that card" and the other revealed cards go — "Put that card
//! onto the battlefield and the rest on the bottom of your library in a random order.",
//! "Put that card into your hand and exile all other cards revealed this way.", "Put that
//! card onto the battlefield tapped and attacking and the rest ...", "..., then shuffle the
//! rest into your library."
//!
//! The first sentence compiles to an [`Effect::RevealUntil`] that leaves the cards where
//! they are ("that card" is `vars::IT`); the second one says where they go.

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::end;

inventory::submit! {
    EffectPattern { name: "card_flow: reveal cards until you reveal a [card]", priority: 90, parse: reveal_until }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: put that card ... and the rest ...", priority: 90, apply: put_that_card_and_the_rest }
}

/// Cards left where they are (not yet told where they go).
fn in_place() -> Destination {
    let mut d = Destination::library_top();
    d.position = LibraryPosition::FromTop(0);
    d
}

fn is_in_place(d: &Destination) -> bool {
    d.zone == ZoneKind::Library && matches!(d.position, LibraryPosition::FromTop(_))
}

/// "reveal cards from the top of your library until you reveal a creature card".
fn reveal_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal cards from the top of your library until you reveal ")?;
    let desc = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let filter = card_filter(desc, b)?;
    // "that card" is the card revealed last.
    b.it = Sel::Var(vars::IT);
    Some(Effect::RevealUntil {
        who: PlayerRef::You,
        filter: Filter::And(vec![Filter::Card, filter]),
        found_to: in_place(),
        rest_to: in_place(),
    })
}

/// Where "that card" goes.
fn found_destination(s: &str) -> Option<(Destination, &str)> {
    let mut attacking = Destination::battlefield().under_your_control().tapped();
    attacking.attacking = true;
    for (p, d) in [
        ("onto the battlefield tapped and attacking", attacking),
        (
            "onto the battlefield tapped",
            Destination::battlefield().under_your_control().tapped(),
        ),
        (
            "onto the battlefield",
            Destination::battlefield().under_your_control(),
        ),
        ("into your hand", Destination::zone(ZoneKind::Hand)),
        ("in your hand", Destination::zone(ZoneKind::Hand)),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((d, r));
        }
    }
    None
}

/// Where the other revealed cards go ("the rest ...", "all other cards revealed this way
/// ...").
fn rest_destination(s: &str) -> Option<Destination> {
    let bottom = |random: bool| {
        let mut d = Destination::library_bottom();
        if random {
            d.position = LibraryPosition::BottomRandom;
        }
        d
    };
    let shuffled = || {
        let mut d = Destination::library_top();
        d.position = LibraryPosition::Shuffled;
        d
    };
    Some(match s {
        " and the rest on the bottom of your library in a random order"
        | " and put all other cards revealed this way on the bottom of your library in a random order"
        | ", then put the rest on the bottom of your library in a random order"
        | ", then put all other cards revealed this way on the bottom of your library in a random order" => {
            bottom(true)
        }
        " and the rest on the bottom of your library in any order"
        | " and put all other cards revealed this way on the bottom of your library in any order" => {
            bottom(false)
        }
        " and the rest into your graveyard"
        | " and all other cards revealed this way into your graveyard"
        | " and put all other cards revealed this way into your graveyard" => {
            Destination::zone(ZoneKind::Graveyard)
        }
        " and exile all other cards revealed this way" => Destination::zone(ZoneKind::Exile),
        " and shuffle the rest into your library" | ", then shuffle the rest into your library" => {
            shuffled()
        }
        _ => return None,
    })
}

/// "put that card onto the battlefield and the rest on the bottom of your library in a
/// random order", after revealing cards until one was found.
fn put_that_card_and_the_rest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("put that card ") else {
        return false;
    };
    let Some((found, r)) = found_destination(r) else {
        return false;
    };
    let Some(rest) = rest_destination(r) else {
        return false;
    };
    let Effect::RevealUntil {
        who: PlayerRef::You,
        found_to,
        rest_to,
        ..
    } = prev
    else {
        return false;
    };
    if !is_in_place(found_to) || !is_in_place(rest_to) {
        return false;
    }
    *found_to = found;
    *rest_to = rest;
    true
}
