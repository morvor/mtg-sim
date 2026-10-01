//! Counting mana values among objects (CR 202.3): the condition "there are five or more
//! mana values among cards in your graveyard" (`Value::ManaValuesAmong`). A land card's
//! mana value is 0, so land cards count toward it (CR 202.3a). The value forms ("the
//! number of different mana values among ...", "for each different mana value among
//! ...") are read with the other value phrases.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

/// "there are five or more mana values among cards in your graveyard", "there are three
/// or more different mana values among nonland permanents you control".
fn mana_values_among_condition(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("there are ")?;
    let (n, rest) = parse_number(r)?;
    let r = rest.trim_start().strip_prefix("or more ")?;
    let r = r.strip_prefix("different ").unwrap_or(r);
    let r = r.strip_prefix("mana values among ")?;
    let (f, true, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(Value::ManaValuesAmong(f), Cmp::Ge, n))
}

inventory::submit! { ConditionPattern { name: "misc: there are N or more mana values among [objects]", priority: 100, parse: mana_values_among_condition } }
