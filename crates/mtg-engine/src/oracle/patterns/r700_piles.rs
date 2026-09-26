//! Piles (CR 700.3) and revealing the top cards of a library:
//!
//! * "Reveal the top five cards of your library." (the cards are "those cards"/"them");
//! * "An opponent separates those cards into two piles." / "... and separate them into
//!   two piles." / "Separate all permanents target player controls into two piles.";
//! * "An opponent chooses one of those piles." and what's done with the piles: "Put one
//!   pile into your hand and the other into your graveyard.", "Put that pile into your
//!   hand and the other into your graveyard.", "That player sacrifices all permanents in
//!   the pile of their choice.", "Destroy all creatures in the pile of that player's
//!   choice.", "Exile the pile of an opponent's choice and return the other to the
//!   battlefield."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::piles::{PileAction, CHOSEN, OTHER};

fn separate(what: Sel, separator: PlayerRef) -> Effect {
    Effect::Piles(Box::new(PileAction::Separate { what, separator }))
}

fn choose(chooser: PlayerRef) -> Effect {
    Effect::Piles(Box::new(PileAction::Choose { chooser }))
}

fn to_zone(var: crate::ability::Var, zone: ZoneKind) -> Effect {
    Effect::Move {
        what: Sel::Var(var),
        to: Destination::zone(zone),
    }
}

/// "reveal the top five cards of your library": they stay where they are, revealed while
/// the rest of the effect needs them (CR 701.20a); "those cards" refers to them.
fn reveal_top_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal the top ")?;
    let (n, r) = parse_number(r)?;
    if end(r) != "cards of your library" {
        return None;
    }
    b.it = Sel::Var(vars::REVEALED);
    Some(Effect::Dig {
        who: PlayerRef::You,
        n,
        reveal: true,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: false,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to: Destination {
            position: LibraryPosition::FromTop(0),
            ..Destination::library_top()
        },
    })
}

inventory::submit! { EffectPattern { name: "r700 reveal the top cards", priority: 100, parse: reveal_top_cards } }

/// "an opponent separates those cards into two piles", "separate them into two piles",
/// "separate all permanents target player controls into two piles".
fn separate_into_piles(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (separator, r) = if let Some(r) = l.strip_prefix("an opponent separates ") {
        (PlayerRef::EachOpponent, r)
    } else if let Some(r) = l.strip_prefix("separate ") {
        (PlayerRef::You, r)
    } else {
        return None;
    };
    let objs = r.strip_suffix(" into two piles")?;
    let what = match objs {
        "those cards" | "them" => b.it.clone(),
        _ => {
            let (sel, tail) = object_ref(objs, b)?;
            if !end(&tail).is_empty() {
                return None;
            }
            sel
        }
    };
    Some(separate(what, separator))
}

inventory::submit! { EffectPattern { name: "r700 separate into piles", priority: 100, parse: separate_into_piles } }

/// "an opponent chooses one of those piles", "that player chooses one of those piles".
fn choose_a_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" chooses one of those piles")?;
    let who = match r {
        "an opponent" => PlayerRef::EachOpponent,
        _ => {
            let (p, tail) = player_ref(r, b)?;
            if !end(&tail).is_empty() {
                return None;
            }
            p
        }
    };
    Some(choose(who))
}

inventory::submit! { EffectPattern { name: "r700 choose a pile", priority: 100, parse: choose_a_pile } }

/// "into your hand", "into your graveyard", "onto the battlefield".
fn pile_destination(s: &str) -> Option<ZoneKind> {
    Some(match end(s) {
        "into your hand" | "into its owner's hand" | "into their owners' hands" => {
            ZoneKind::Hand
        }
        "into your graveyard" | "into its owner's graveyard" | "into their owners' graveyards" => {
            ZoneKind::Graveyard
        }
        "onto the battlefield" | "to the battlefield" => ZoneKind::Battlefield,
        _ => return None,
    })
}

/// "put one pile into your hand and the other into your graveyard" (you choose), "put
/// that pile into your hand and the other into your graveyard" (already chosen).
fn put_piles(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let (pick, r) = if let Some(r) = r.strip_prefix("one pile ") {
        (true, r)
    } else if let Some(r) = r.strip_prefix("that pile ") {
        (false, r)
    } else {
        return None;
    };
    let (first, second) = r.split_once(" and the other ")?;
    let first = pile_destination(first)?;
    let second = pile_destination(second)?;
    let mut out = Vec::new();
    if pick {
        out.push(choose(PlayerRef::You));
    }
    out.push(to_zone(CHOSEN, first));
    out.push(to_zone(OTHER, second));
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "r700 put piles", priority: 100, parse: put_piles } }

/// "that player sacrifices all permanents in the pile of their choice", "destroy all
/// creatures in the pile of that player's choice", "exile the pile of an opponent's choice
/// and return the other to the battlefield".
fn pile_of_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "[player] sacrifices all [objects] in the pile of their choice"
    if let Some((who, r)) = l.split_once(" sacrifices all ") {
        let (_, _, tail) = parse_object_phrase(r)?;
        if end(tail) != "in the pile of their choice" {
            return None;
        }
        let (p, rest) = player_ref(who, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        return Some(Effect::seq(vec![
            choose(p),
            Effect::SacrificeObjects {
                what: Sel::Var(CHOSEN),
            },
        ]));
    }
    // "destroy all [objects] in the pile of that player's choice"
    if let Some(r) = l.strip_prefix("destroy all ") {
        let (_, _, tail) = parse_object_phrase(r)?;
        let who = end(tail)
            .strip_prefix("in the pile of ")?
            .strip_suffix("'s choice")?;
        let (p, rest) = player_ref(who, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        return Some(Effect::seq(vec![
            choose(p),
            Effect::Destroy {
                what: Sel::Var(CHOSEN),
                no_regen: false,
            },
        ]));
    }
    // "exile the pile of an opponent's choice and return the other to the battlefield"
    if let Some(r) = l.strip_prefix("exile the pile of an opponent's choice and return the other ") {
        let zone = pile_destination(r)?;
        return Some(Effect::seq(vec![
            choose(PlayerRef::EachOpponent),
            Effect::Exile {
                what: Sel::Var(CHOSEN),
                face_down: false,
                link: false,
            },
            to_zone(OTHER, zone),
        ]));
    }
    None
}

inventory::submit! { EffectPattern { name: "r700 pile of their choice", priority: 100, parse: pile_of_choice } }
