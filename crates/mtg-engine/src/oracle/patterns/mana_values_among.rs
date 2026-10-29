//! Counting different mana values among objects (CR 202.3): "the number of different mana
//! values among cards in your graveyard", "for each different mana value among instant
//! and sorcery cards in your graveyard" (`Value::ManaValuesAmong`), and the condition
//! "there are five or more mana values among cards in your graveyard". Each card has one
//! mana value: a split card's is its halves' total (CR 202.3d, 709.4), a double-faced
//! card's is its front face's (CR 202.3b, 712.8a), a land card's is 0 and X is 0 off the
//! stack (CR 202.3e).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

/// "[different] mana value(s) among [objects]" → the value and the text after the
/// objects.
pub(crate) fn value(s: &str) -> Option<(Value, &str)> {
    let s = s.trim_start();
    let s = s.strip_prefix("different ").unwrap_or(s);
    let r = s
        .strip_prefix("mana values among ")
        .or_else(|| s.strip_prefix("mana value among "))?;
    let (f, true, rest) = parse_object_phrase(r)? else {
        return None;
    };
    Some((Value::ManaValuesAmong(f), rest))
}

/// "there are N or more [different] mana values among [objects]".
fn mana_values_condition(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("there are ")?;
    let (n, rest) = parse_number(r)?;
    let rest = rest.trim_start().strip_prefix("or more ")?;
    let (v, tail) = value(rest)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(v, Cmp::Ge, n))
}

inventory::submit! { ConditionPattern { name: "r202.3: there are N or more mana values among [objects]", priority: 100, parse: mana_values_condition } }
