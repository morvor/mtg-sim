//! "as long as you own a card in exile that has an Adventure" (Howling Galefang): a
//! condition true while an object matching the phrase that you own exists (CR 108.3).

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::{end, parse_object_phrase};

fn you_own_a(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you own ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() || f.zone().is_none() {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        f,
        Filter::OwnedBy(PlayerRel::You),
    ])))
}

inventory::submit! {
    ConditionPattern {
        name: "you own a [card in a zone]",
        priority: 100,
        parse: you_own_a,
    }
}
