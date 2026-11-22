//! "Exile the top X cards of your library. You may cast instant and sorcery spells with
//! mana value X or less from among them without paying their mana costs. Then put all
//! cards exiled this way that weren't cast into your graveyard." (Epic Experiment), "You
//! may cast any number of spells with mana value 5 or less from among them without paying
//! their mana costs." (Hazoret's Undying Fury): as the effect resolves, its controller
//! chooses any number of the exiled cards with that quality and casts them one after the
//! other (CR 608.2g) without paying their mana costs (CR 118.9; X is 0, CR 107.3b),
//! ignoring timing permissions based on their types but following other timing
//! restrictions. The exiled cards are remembered in [`EXILED`] for "cards exiled this way
//! that weren't cast".

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::CardType;

/// The cards the effect exiled, before any is cast.
const EXILED: Var = vars::USER + 1774;

/// Whether the effect ends by exiling the top cards of a library.
fn ends_with_exile_top(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(..),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile_top),
        _ => false,
    }
}

fn cast_any_from_among(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l.trim())
        .strip_prefix("you may cast ")
        .and_then(|r| {
            ["them", "those cards", "the exiled cards"].iter().find_map(|w| {
                r.strip_suffix(&format!(
                    " from among {w} without paying their mana costs"
                ))
            })
        })
    else {
        return false;
    };
    let r = r.strip_prefix("any number of ").unwrap_or(r);
    // "[quality] spells [with ...]": the cards have that quality.
    let Some((before, after)) = r.split_once("spells") else {
        return false;
    };
    let desc = format!("{before}cards{after}");
    let quality = match parse_object_phrase(desc.trim()) {
        Some((f, _, tail)) if end(tail).trim().is_empty() => f,
        _ => return false,
    };
    if !ends_with_exile_top(prev) {
        return false;
    }
    let cast = Effect::CastCard {
        who: PlayerRef::You,
        // Any number, none included ("you may").
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::In(Box::new(Sel::Var(EXILED))),
                Filter::InZone(ZoneKind::Exile),
                Filter::Not(Box::new(Filter::Type(CardType::Land))),
                quality,
            ]),
            count: Value::c(99),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    };
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::Store {
            var: EXILED,
            sel: Sel::Var(vars::IT),
        },
        cast,
    ]);
    true
}

/// Whether the effect remembered the exiled cards in [`EXILED`].
fn remembers_exiled(e: &Effect) -> bool {
    match e {
        Effect::Store { var, .. } => *var == EXILED,
        Effect::Seq(v) => v.iter().any(remembers_exiled),
        _ => false,
    }
}

/// "Then put all cards exiled this way that weren't cast into your graveyard."
fn put_the_rest_into_graveyard(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l.trim());
    let l = l.strip_prefix("then ").unwrap_or(l);
    if !matches!(
        l,
        "put all cards exiled this way that weren't cast into your graveyard"
            | "put the exiled cards that weren't cast this way into your graveyard"
    ) || !remembers_exiled(prev)
    {
        return false;
    }
    let rest = Sel::All(Filter::and(vec![
        Filter::In(Box::new(Sel::Var(EXILED))),
        Filter::InZone(ZoneKind::Exile),
    ]));
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::Move {
            what: rest,
            to: Destination::zone(ZoneKind::Graveyard),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "you may cast [quality] spells from among the exiled cards without paying their mana costs", priority: 90, apply: cast_any_from_among } }
inventory::submit! { FollowupPattern { name: "then put all cards exiled this way that weren't cast into your graveyard", priority: 90, apply: put_the_rest_into_graveyard } }
