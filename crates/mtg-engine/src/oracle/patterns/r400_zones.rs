//! Oracle patterns for the zones (CR 400–406): playing with the top card of a library
//! revealed (CR 401.5), putting objects into their owners' libraries at a position
//! (CR 400.3, 401.4, 401.7), and shuffling a whole zone into a library (CR 400.12).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

/// "Play with the top card of your library revealed." / "Players play with the top card of
/// their libraries revealed." (CR 401.5).
fn reveal_top(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rel = match end(l) {
        "play with the top card of your library revealed" => PlayerRel::You,
        "players play with the top card of their libraries revealed"
        | "each player plays with the top card of their library revealed" => PlayerRel::Any,
        _ => return None,
    };
    Some(vec![static_ability(StaticEffect::RevealTopCard(rel), text)])
}

inventory::submit! { StaticPattern { name: "r401 play with the top card revealed", priority: 100, parse: reveal_top } }

/// "second" → 2, ... ("Nth from the top").
fn ordinal(w: &str) -> Option<u32> {
    Some(match w {
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        "tenth" => 10,
        _ => return None,
    })
}

/// Where in its owner's library: "on top of its owner's library", "on the bottom of their
/// owners' libraries", "into its owner's library third from the top".
fn library_destination(rest: &str) -> Option<Destination> {
    let rest = end(rest);
    let owners = [
        "its owner's library",
        "their owner's library",
        "their owners' libraries",
    ];
    if let Some(r) = rest.strip_prefix("on top of ") {
        return owners.contains(&r).then(Destination::library_top);
    }
    if let Some(r) = rest.strip_prefix("on the bottom of ") {
        return owners.contains(&r).then(Destination::library_bottom);
    }
    let r = rest.strip_prefix("into ")?;
    let owner = owners.iter().find(|o| r.starts_with(**o))?;
    let r = r[owner.len()..].trim();
    let n = ordinal(r.strip_suffix(" from the top")?)?;
    let mut d = Destination::zone(ZoneKind::Library);
    // 0-based; a library with fewer cards gets it on the bottom (CR 401.7).
    d.position = LibraryPosition::FromTop(n - 1);
    Some(d)
}

/// "put target creature on top of its owner's library", "put all creatures on the bottom
/// of their owners' libraries", "put target spell or nonland permanent into its owner's
/// library second from the top".
fn put_into_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let saved_targets = b.targets.len();
    let saved_it = b.it.clone();
    if let Some((what, rest)) = object_ref(r, b) {
        if let Some(to) = library_destination(&rest) {
            return Some(Effect::Move { what, to });
        }
    }
    b.targets.truncate(saved_targets);
    b.it = saved_it;
    None
}

inventory::submit! { EffectPattern { name: "r401 put into its owner's library", priority: 100, parse: put_into_library } }

fn cards_of(zone: ZoneKind, who: PlayerRel) -> Sel {
    Sel::All(Filter::and(vec![
        Filter::Card,
        Filter::InZone(zone),
        Filter::OwnedBy(who),
    ]))
}

/// "each player shuffles their hand and graveyard into their library[, then draws seven
/// cards]", "shuffle your hand and graveyard into your library", "each player shuffles
/// their graveyard into their library": the action is performed on every card in the
/// zones (CR 400.12).
fn shuffle_zones_into_library(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "..., then draws seven cards" (Timetwister).
    let (l, draw) = match l.split_once(", then draw") {
        Some((a, d)) => {
            let d = d.strip_prefix('s').unwrap_or(d).trim();
            let (n, rest) = crate::oracle::phrases::parse_card_count(d)?;
            if !end(rest).is_empty() {
                return None;
            }
            (a, Some(n))
        }
        None => (l, None),
    };
    let (each, r) = if let Some(r) = l.strip_prefix("each player shuffles their ") {
        (true, r)
    } else {
        (false, l.strip_prefix("shuffle your ")?)
    };
    let (zones, lib) = r.split_once(" into ")?;
    if lib != if each { "their library" } else { "your library" } {
        return None;
    }
    let who = if each {
        PlayerRel::Iterated
    } else {
        PlayerRel::You
    };
    let sels: Vec<Sel> = match zones {
        "hand and graveyard" | "graveyard and hand" => vec![
            cards_of(ZoneKind::Hand, who),
            cards_of(ZoneKind::Graveyard, who),
        ],
        "hand" => vec![cards_of(ZoneKind::Hand, who)],
        "graveyard" => vec![cards_of(ZoneKind::Graveyard, who)],
        _ => return None,
    };
    let what = if sels.len() == 1 {
        sels.into_iter().next()?
    } else {
        Sel::Union(sels)
    };
    let mut e = Effect::ShuffleInto { what };
    if let Some(n) = draw {
        let who = if each {
            PlayerRef::Iterated
        } else {
            PlayerRef::You
        };
        e = Effect::seq(vec![e, Effect::Draw { who, n }]);
    }
    Some(if each {
        Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

inventory::submit! { EffectPattern { name: "r400 shuffle hand and graveyard into library", priority: 100, parse: shuffle_zones_into_library } }
