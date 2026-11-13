//! "you control a Desert or there is a Desert card in your graveyard" (the Deserts of
//! Amonkhet): either condition is enough, and having more than one of them doesn't
//! matter.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

/// "a[n] [object phrase]" → its filter, when nothing follows it.
fn single(s: &str) -> Option<Filter> {
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    (!plural && end(tail).is_empty()).then_some(f)
}

fn control_or_card_in_graveyard(c: &str) -> Option<Condition> {
    let (a, b) = end(c).split_once(" or there is ")?;
    let controlled = single(a.strip_prefix("you control ")?)?;
    let card = single(b.strip_suffix(" in your graveyard")?)?;
    Some(Condition::Or(vec![
        Condition::Exists(controlled.you_control()),
        Condition::Compare(
            Value::CardsInGraveyard(PlayerRef::You, card),
            Cmp::Ge,
            Value::c(1),
        ),
    ]))
}

inventory::submit! { ConditionPattern { name: "you control a [type] or there is a [type] card in your graveyard", priority: 100, parse: control_or_card_in_graveyard } }
