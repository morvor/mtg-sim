//! "[Player] mills N cards. You may cast a[n] [quality] spell from among them without
//! paying its mana cost." (Jace's Mindseeker): the cards milled are "them" (the mill
//! effect names them, see `Effect::Mill`); casting one is part of the resolving ability
//! (CR 608.2g), with any {X} in its mana cost 0 (CR 107.3b, 118.9a).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

/// Whether the effect ends with milling cards.
fn ends_with_mill(e: &Effect) -> bool {
    match e {
        Effect::Mill { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_mill),
        _ => false,
    }
}

fn cast_from_among_milled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some(r) = l
        .strip_prefix("you may cast ")
        .and_then(|r| r.strip_suffix(" from among them without paying its mana cost"))
        .and_then(|r| r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")))
        .and_then(|r| r.strip_suffix("spell"))
    else {
        return false;
    };
    let desc = r.trim();
    let quality = if desc.is_empty() {
        Filter::Any
    } else {
        match parse_object_phrase(desc) {
            Some((f, _, tail)) if end(tail).trim().is_empty() => f,
            _ => return false,
        }
    };
    if !ends_with_mill(prev) {
        return false;
    }
    let cast = Effect::CastCard {
        who: PlayerRef::You,
        // Choosing none is not casting one ("you may").
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![Filter::In(Box::new(Sel::Var(vars::IT))), quality]),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    };
    let milled = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::Seq(vec![milled, cast]);
    true
}

inventory::submit! { FollowupPattern { name: "you may cast a [quality] spell from among the milled cards without paying its mana cost", priority: 100, apply: cast_from_among_milled } }
