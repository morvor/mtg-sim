//! Qualities of objects named after the noun of an object phrase:
//!
//! - "with {X} in their mana cost" (CR 107.3, 202.1);
//! - "that has an activated ability with {T} in its cost" (CR 602.1, 107.5);
//! - "that doesn't have a +1/+1 counter on it", "that don't have counters on them";
//! - "that didn't attack or enter this turn", "that attacked or entered this turn";
//! - "that don't have a name", "that doesn't have a name" (CR 201.2a, 708.2a);
//! - "with two or more card types" (CR 205.2a);
//! - "with a legendary Equipment attached to it" (CR 301.5);
//! - "whose controller controls an Island";
//! - "that doesn't have the same name as a token you control", "that doesn't have the
//!   same name as another permanent you control" (CR 201.2).
//!
//! Each quality is checked on the object the phrase describes ([`vars::TESTED`] in
//! [`Filter::ValueCmp`]) or by a named predicate in `kw/filter_qualities_rules.rs`.

use super::FilterSuffixPattern;
use crate::ability::*;
use crate::oracle::patterns::statics_conditions::counters_on_it;
use crate::oracle::phrases::*;

/// `Filter::Custom`: an object with no name (CR 201.2a; a face-down permanent, CR 708.2a).
pub const NO_NAME: &str = "has no name";
/// `Filter::Custom`: an object with an activated ability with {T} in its cost.
pub const TAP_ABILITY: &str = "has an activated ability with {T} in its cost";
/// `Filter::Custom`: an object that has a color that's the most common color among all
/// permanents or tied for most common (each permanent counts once for each of its
/// colors, CR 105.2).
pub const MOST_COMMON_COLOR: &str = "shares a color with the most common color among all permanents";
/// `Filter::Custom`: an object with two or more card types.
pub const TWO_CARD_TYPES: &str = "has two or more card types";

fn tested() -> Sel {
    Sel::Var(vars::TESTED)
}

/// A filter that's true when any object matches `f` (evaluated with the tested object
/// bound, so `f` may mention it).
fn any_matching(f: Filter) -> Filter {
    Filter::ValueCmp(Box::new(Value::Count(f)), Cmp::Ge, Box::new(Value::c(1)))
}

fn qualities<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for p in [
        "with {x} in their mana costs",
        "with {x} in their mana cost",
        "with {x} in its mana cost",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::HasX, r));
        }
    }
    for p in [
        "that has an activated ability with {t} in its cost",
        "that have an activated ability with {t} in its cost",
        "with an activated ability with {t} in its cost",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::Custom(TAP_ABILITY.into()), r));
        }
    }
    for p in [
        "that don't have a name",
        "that doesn't have a name",
        "without a name",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((Filter::Custom(NO_NAME.into()), r));
        }
    }
    if let Some(r) = t.strip_prefix("with two or more card types") {
        return Some((Filter::Custom(TWO_CARD_TYPES.into()), r));
    }
    // "that doesn't have a +1/+1 counter on it", "that don't have counters on them"
    for p in ["that doesn't have ", "that don't have ", "without "] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        for end_ in [" on it", " on them"] {
            let Some(i) = x.find(end_) else {
                continue;
            };
            let body = &x[..i];
            let r = &x[i + end_.len()..];
            let body = body
                .strip_prefix("any ")
                .map(|k| format!("a {}", k.trim_end_matches('s')))
                .unwrap_or_else(|| body.to_string());
            let (cmp, n, kind) = counters_on_it(&format!("{body} on it"))?;
            if cmp != Cmp::Ge || !matches!(n, Value::Const(1)) {
                return None;
            }
            return Some((Filter::not(Filter::HasCounter(kind)), r));
        }
    }
    // "that didn't attack or enter this turn", "that attacked or entered this turn"
    for (p, neg) in [("that didn't ", true), ("that attacked or entered", false)] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        let r = if neg {
            x.strip_prefix("attack or enter this turn")?
        } else {
            x.strip_prefix(" this turn")?
        };
        let f = Filter::Or(vec![Filter::AttackedThisTurn, Filter::EnteredThisTurn]);
        return Some((if neg { Filter::not(f) } else { f }, r));
    }
    // "with a legendary Equipment attached to it"
    if let Some(x) = t.strip_prefix("with a ").or_else(|| t.strip_prefix("with an ")) {
        if let Some(i) = x.find(" attached to it") {
            let (f, plural, rest) = parse_object_phrase(&x[..i])?;
            if plural || !rest.trim().is_empty() {
                return None;
            }
            let attached = Filter::and(vec![f, Filter::AttachedTo(Box::new(tested()))]);
            return Some((any_matching(attached), &x[i + " attached to it".len()..]));
        }
    }
    // "whose controller controls an Island"
    if let Some(x) = t.strip_prefix("whose controller controls ") {
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))?;
        let (f, plural, rest) = parse_object_phrase(x)?;
        if plural {
            return None;
        }
        return Some((
            Filter::ControllerMatches(Box::new(PlayerFilter::Controls(
                Box::new(f),
                Cmp::Ge,
                Box::new(Value::c(1)),
            ))),
            rest,
        ));
    }
    // "that doesn't have the same name as a token you control", "... as another
    // permanent you control": no such other object has a name in common with it.
    for p in [
        "that doesn't have the same name as ",
        "that don't have the same name as ",
    ] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an ")).unwrap_or(x);
        let (f, plural, rest) = parse_object_phrase(x)?;
        if plural {
            return None;
        }
        // "another": other than the object described (not the source).
        let f = other_than_tested(f);
        let same = Filter::and(vec![
            f,
            Filter::SameNameAs(Box::new(tested())),
            Filter::not(Filter::In(Box::new(tested()))),
        ]);
        return Some((Filter::not(any_matching(same)), rest));
    }
    None
}

fn other_than_tested(f: Filter) -> Filter {
    match f {
        Filter::Other => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(other_than_tested).collect()),
        f => f,
    }
}

inventory::submit! { FilterSuffixPattern { name: "filter qualities", priority: 120, parse: qualities } }
