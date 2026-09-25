//! Looking at and revealing the top cards of a library: "Look at the top N cards of your
//! library." followed by what happens to them — "Put one of them into your hand and the
//! rest on the bottom of your library in any order.", "You may reveal a creature card
//! from among them and put it into your hand. Put the rest on the bottom of your library
//! in a random order.", "then put them back in any order", "Put the rest into your
//! graveyard."
//!
//! The first sentence compiles to an [`Effect::Dig`] that takes nothing and leaves the
//! cards where they are (`LibraryPosition::FromTop`); the following sentences are
//! follow-ups that fill in what's taken and where the rest go.

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{EffectPattern, FollowupPattern};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "card_flow: look at / reveal the top N cards", priority: 90, parse: look_at_top }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: put them back in any order", priority: 90, apply: put_them_back }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: put N of them into your hand and the rest ...", priority: 90, apply: put_some_and_rest }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: you may put a [card] from among them ...", priority: 90, apply: take_from_among }
}
inventory::submit! {
    FollowupPattern { name: "card_flow: put the rest ...", priority: 90, apply: put_the_rest }
}

/// Cards left where they are (not yet told where the rest go).
fn in_place() -> Destination {
    let mut d = Destination::library_top();
    d.position = LibraryPosition::FromTop(0);
    d
}

fn is_in_place(d: &Destination) -> bool {
    d.zone == ZoneKind::Library && matches!(d.position, LibraryPosition::FromTop(0))
}

/// "look at the top N cards of your library", "reveal the top card of your library".
fn look_at_top(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (reveal, r) = if let Some(r) = l.strip_prefix("look at the top ") {
        (false, r)
    } else if let Some(r) = l.strip_prefix("reveal the top ") {
        (true, r)
    } else {
        return None;
    };
    let (n, r) = if let Some(r) = r.strip_prefix("card of ") {
        (Value::c(1), r)
    } else {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (n, r.strip_prefix("cards of ")?)
    };
    let who = match r {
        "your library" => PlayerRef::You,
        "target player's library" | "target opponent's library" => {
            // Only looking (nothing is taken) works for another player's library: the
            // player looking isn't the library's owner.
            let (pf, text) = if r.starts_with("target player") {
                (PlayerFilter::Any, "target player")
            } else {
                (PlayerFilter::Opponent, "target opponent")
            };
            let slot = b.add_target(TargetSpec::player(pf, text), text);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::Dig {
        who,
        n,
        reveal,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: true,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to: in_place(),
    })
}

/// The effect is a look at your own library with nothing decided yet.
fn look_only(prev: &mut Effect) -> Option<&mut Effect> {
    match prev {
        Effect::Dig {
            who: PlayerRef::You,
            take: Value::Const(0),
            rest_to,
            ..
        } if is_in_place(rest_to) => Some(prev),
        _ => None,
    }
}

/// "put them back in any order" (also after ", then").
fn put_them_back(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = l.strip_prefix("then ").unwrap_or(l);
    if !matches!(
        l,
        "put them back in any order" | "put those cards back in any order"
    ) {
        return false;
    }
    let Some(Effect::Dig { rest_to, .. }) = look_only(prev) else {
        return false;
    };
    *rest_to = Destination::library_top();
    true
}

/// Where the rest go: "on the bottom of your library in any order", "... in a random
/// order", "on the bottom of your library" (one card), "into your graveyard", "back on top
/// of your library in any order".
fn rest_destination(s: &str, single: bool) -> Option<Destination> {
    let s = s.trim();
    Some(match s {
        "on the bottom of your library in any order" => Destination::library_bottom(),
        "on the bottom of your library in a random order" => {
            let mut d = Destination::library_bottom();
            d.position = LibraryPosition::BottomRandom;
            d
        }
        "on the bottom of your library" if single => Destination::library_bottom(),
        "on top of your library" | "back on top of your library" if single => {
            Destination::library_top()
        }
        "back on top of your library in any order" | "on top of your library in any order" => {
            Destination::library_top()
        }
        "into your graveyard" => Destination::zone(ZoneKind::Graveyard),
        _ => return None,
    })
}

/// Where the chosen cards go.
fn take_destination(s: &str) -> Option<(Destination, &str)> {
    for (p, d) in [
        (
            "into your hand",
            Destination::zone(ZoneKind::Hand) as Destination,
        ),
        ("into your graveyard", Destination::zone(ZoneKind::Graveyard)),
        (
            "onto the battlefield tapped",
            Destination::battlefield().under_your_control().tapped(),
        ),
        (
            "onto the battlefield",
            Destination::battlefield().under_your_control(),
        ),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((d, r));
        }
    }
    None
}

/// "one of them", "two of them", "up to two of them", "one of those cards".
fn count_of_them(s: &str) -> Option<(Value, bool, &str)> {
    let (up_to, s) = match s.strip_prefix("up to ") {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (n, r) = parse_number(s)?;
    n.as_const()?;
    let r = r
        .strip_prefix("of them ")
        .or_else(|| r.strip_prefix("of those cards "))?;
    Some((n, up_to, r))
}

/// "put one of them into your hand and the rest on the bottom of your library in any
/// order", "put one of them into your hand and the other into your graveyard", "put up to
/// two of them into your hand and the rest into your graveyard".
fn put_some_and_rest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("put ") else {
        return false;
    };
    let Some((k, up_to, r)) = count_of_them(r) else {
        return false;
    };
    let Some((to, r)) = take_destination(r) else {
        return false;
    };
    let (single, r) = if let Some(x) = r.strip_prefix(" and the other ") {
        (true, x)
    } else if let Some(x) = r.strip_prefix(" and the rest ") {
        (false, x)
    } else {
        return false;
    };
    let Some(Effect::Dig {
        n,
        take,
        take_up_to,
        take_to,
        rest_to,
        ..
    }) = look_only(prev)
    else {
        return false;
    };
    // "the other": exactly one card is left.
    if single {
        match (n.as_const(), k.as_const()) {
            (Some(n), Some(k)) if n == k + 1 && !up_to => {}
            _ => return false,
        }
    }
    let Some(rest) = rest_destination(r, single) else {
        return false;
    };
    // The chosen cards can't go where the rest go (e.g. both into the graveyard).
    if rest.zone == to.zone {
        return false;
    }
    *take = k;
    *take_up_to = up_to;
    *take_to = to;
    *rest_to = rest;
    true
}

/// "you may reveal a creature card from among them and put it into your hand", "you may
/// put a land card from among them onto the battlefield tapped", "you may reveal up to two
/// creature cards from among them and put them into your hand", "put a creature card from
/// among them into your hand".
fn take_from_among(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (may, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let (reveal, r) = if let Some(r) = r.strip_prefix("reveal ") {
        (true, r)
    } else if let Some(r) = r.strip_prefix("put ") {
        (false, r)
    } else {
        return false;
    };
    // Count.
    let (k, up_to, r) = if let Some(r) = r.strip_prefix("any number of ") {
        (Value::c(999), true, r)
    } else if let Some(r) = r.strip_prefix("up to ") {
        let Some((n, r)) = parse_number(r) else {
            return false;
        };
        (n, true, r)
    } else {
        let Some((n, r)) = parse_number(r) else {
            return false;
        };
        (n, may, r)
    };
    if k.as_const().is_none() {
        return false;
    }
    let Some((desc, r)) = r.split_once(" from among them") else {
        return false;
    };
    let Some(filter) = card_filter(desc, b) else {
        return false;
    };
    let r = r.trim_start();
    let dest = if reveal {
        let Some(x) = ["and put it ", "and put that card ", "and put them "]
            .iter()
            .find_map(|p| r.strip_prefix(p))
        else {
            return false;
        };
        x
    } else {
        r
    };
    let Some((to, tail)) = take_destination(dest) else {
        return false;
    };
    // Revealed cards go to a hidden zone; putting a card onto the battlefield needs no reveal.
    if !tail.is_empty() || (reveal && to.zone != ZoneKind::Hand) {
        return false;
    }
    let Some(Effect::Dig {
        filter: f,
        take,
        take_up_to,
        take_to,
        ..
    }) = look_only(prev)
    else {
        return false;
    };
    *f = filter;
    *take = k;
    *take_up_to = up_to;
    *take_to = to;
    true
}

/// "put the rest on the bottom of your library in a random order", "put the rest into
/// your graveyard", after cards were taken from among them.
fn put_the_rest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = l
        .strip_prefix("put the rest ")
        .or_else(|| l.strip_prefix("then put the rest "))
    else {
        return false;
    };
    let Some(rest) = rest_destination(r, false) else {
        return false;
    };
    match prev {
        Effect::Dig {
            who: PlayerRef::You,
            take,
            take_to,
            rest_to,
            ..
        } if is_in_place(rest_to)
            && !matches!(take, Value::Const(0))
            && take_to.zone != rest.zone =>
        {
            *rest_to = rest;
            true
        }
        _ => false,
    }
}
