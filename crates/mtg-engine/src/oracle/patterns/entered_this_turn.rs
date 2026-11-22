//! Conditions about permanents that entered the battlefield this turn, whether or not
//! they're still there: "two or more nonland permanents entered the battlefield under your
//! control this turn" (celebration), "an artifact entered the battlefield under your
//! control this turn", "another creature entered the battlefield under your control this
//! turn". They look at past events, not the current game state: each permanent as it
//! entered ([`Value::PermanentsEnteredThisTurn`], recorded in
//! `TurnHistory::permanents_entered`), e.g. a face-down creature that has since been
//! turned face up still entered face down.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

/// "[a | an | another | one or more | N or more] [objects] entered the battlefield under
/// your control this turn".
fn entered_under_your_control(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_suffix(" entered the battlefield under your control this turn")
        .or_else(|| c.strip_suffix(" entered under your control this turn"))?;
    // "another creature": one other than the source.
    let (n, phrase) = if r.starts_with("another ") {
        (Value::c(1), r)
    } else {
        let (n, rest) = parse_number(r)?;
        let rest = rest.trim_start();
        match n.as_const()? {
            1 => (n, rest.strip_prefix("or more ").unwrap_or(rest)),
            _ => (n, rest.strip_prefix("or more ")?),
        }
    };
    let (f, _plural, tail) = parse_object_phrase(phrase)?;
    // ("permanent" means one on the battlefield: each is looked at as it entered.)
    if !tail.trim().is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some(Condition::Compare(
        Value::PermanentsEnteredThisTurn(PlayerRef::You, f),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { ConditionPattern { name: "entered the battlefield under your control this turn", priority: 100, parse: entered_under_your_control } }

/// "you had [a land] enter the battlefield under your control this turn" (landfall:
/// Groundswell, Mysteries of the Deep): the same look at past events as
/// "[a land] entered the battlefield under your control this turn".
fn you_had_enter_under_your_control(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("you had ")?
        .strip_suffix(" enter the battlefield under your control this turn")?;
    entered_under_your_control(&format!(
        "{r} entered the battlefield under your control this turn"
    ))
}

inventory::submit! { ConditionPattern { name: "you had [objects] enter the battlefield under your control this turn", priority: 100, parse: you_had_enter_under_your_control } }
