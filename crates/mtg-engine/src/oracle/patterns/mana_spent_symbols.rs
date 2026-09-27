//! "if {G}{G} was spent to cast it": at least that much mana of each color shown was spent
//! to pay the object's total cost (CR 601.2h; the Lorwyn Eclipsed Incarnations, e.g.
//! Wistfulness: "When this creature enters, if {G}{G} was spent to cast it, ..."). The
//! amount is counted by the `mana_spent_of:X` value, which a copy of a spell, or a
//! permanent that wasn't cast, sees as zero (CR 707.10).

use super::ConditionPattern;
use crate::ability::*;

/// The conditions for "[mana symbols] was spent to cast it": one per color shown, at least
/// as many mana of that color as symbols.
fn symbols_spent(c: &str) -> Option<Condition> {
    let c = c.trim().trim_end_matches('.');
    let body = [
        " was spent to cast it",
        " was spent to cast ~",
        " was spent to cast this spell",
        " was spent to cast this creature",
    ]
    .iter()
    .find_map(|s| c.strip_suffix(s))?;
    let inner = body.strip_prefix('{')?.strip_suffix('}')?;
    let mut counts: Vec<(char, i32)> = Vec::new();
    for sym in inner.split("}{") {
        let mut chars = sym.chars();
        let letter = chars.next()?.to_ascii_uppercase();
        if chars.next().is_some() || !"WUBRGC".contains(letter) {
            return None;
        }
        match counts.iter_mut().find(|(l, _)| *l == letter) {
            Some((_, n)) => *n += 1,
            None => counts.push((letter, 1)),
        }
    }
    let mut conds: Vec<Condition> = counts
        .into_iter()
        .map(|(l, n)| {
            Condition::Compare(
                Value::Custom(format!("mana_spent_of:{l}").into()),
                Cmp::Ge,
                Value::c(n),
            )
        })
        .collect();
    Some(if conds.len() == 1 {
        conds.remove(0)
    } else {
        Condition::And(conds)
    })
}

inventory::submit! {
    ConditionPattern { name: "{G}{G} was spent to cast it", priority: 100, parse: symbols_spent }
}
