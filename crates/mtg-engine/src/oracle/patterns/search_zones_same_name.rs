//! Searching a player's graveyard, hand, and library for every card with the name of an
//! object the text targeted (CR 701.23, 201.2):
//!
//! * "Exile target nonblack creature. Search its controller's graveyard, hand, and library
//!   for all cards with the same name as that creature and exile them. Then that player
//!   shuffles." (Eradicate, Splinter, Sowing Salt, Scour)
//! * "Counter target spell. Search its controller's graveyard, hand, and library for all
//!   cards with the same name as that spell and exile them. Then that player shuffles."
//!   (Counterbore, Quash)
//!
//! The name is the target's as it last existed (it has usually left its zone by then), and
//! only those three zones are searched: other objects with that name on the battlefield
//! stay where they are. Every such card in the graveyard is found; in the hand and the
//! library (hidden zones) the searching player may leave some (CR 701.23b). See
//! `kw/search_same_name.rs`.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::search_same_name::{parse_search, search_any_effect, search_effect};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The cards found by the search.
const FOUND: Var = vars::USER + 2311;

fn search_same_name(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("search ")?;
    // "its owner's graveyard, ...": a target card in a graveyard, whose owner is the
    // player whose zones are searched (a card outside the battlefield and the stack is
    // controlled by no one, and is looked at through its owner, CR 108.4a).
    let in_graveyard = b.targets.last().is_some_and(|t| {
        matches!(&t.what, TargetKind::Object(f) if f.zone() == Some(ZoneKind::Graveyard))
    });
    let r = r.strip_prefix("its controller's ").or_else(|| {
        in_graveyard
            .then(|| r.strip_prefix("its owner's "))
            .flatten()
    })?;
    let r = r.strip_prefix("graveyard, hand, and library for ")?;
    // "any number of cards": the searcher may leave some in the graveyard too.
    let (any, r) = if let Some(x) = r.strip_prefix("all cards with the same name as that ") {
        (false, x)
    } else {
        (true, r.strip_prefix("any number of cards with the same name as that ")?)
    };
    let noun = r.strip_suffix(" and exile them")?;
    if noun.contains(' ') || noun.is_empty() {
        return None;
    }
    // "that creature", "that spell": the object the text targeted (the last target).
    let slot = b.targets.len().checked_sub(1)? as u8;
    if !matches!(b.targets[slot as usize].what, TargetKind::Object(_)) {
        return None;
    }
    // The cards are in the zones of the target's controller (as it last existed).
    b.it_player = PlayerRef::ControllerOf(Box::new(Sel::Target(slot)));
    Some(Effect::seq(vec![
        if any {
            search_any_effect(slot, FOUND)
        } else {
            search_effect(slot, FOUND)
        },
        Effect::Exile {
            what: Sel::Var(FOUND),
            face_down: false,
            link: false,
        },
    ]))
}

/// The slot of the target whose name a same-name search (see [`search_same_name`]) uses.
fn same_name_search_slot(e: &Effect) -> Option<u8> {
    match e {
        Effect::Custom(n) => parse_search(n).map(|(slot, _)| slot),
        Effect::Seq(v) => v.iter().rev().find_map(same_name_search_slot),
        _ => None,
    }
}

/// "Then that player shuffles." after the search.
fn then_shuffles(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(l, "then that player shuffles" | "that player shuffles") {
        return false;
    }
    let Some(slot) = same_name_search_slot(prev) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Shuffle {
            who: PlayerRef::ControllerOf(Box::new(Sel::Target(slot))),
        },
    ]);
    true
}

inventory::submit! {
    EffectPattern { name: "search graveyard, hand, and library for cards with the same name", priority: 100, parse: search_same_name }
}
inventory::submit! {
    FollowupPattern { name: "then that player shuffles (after a same-name search)", priority: 100, apply: then_shuffles }
}
