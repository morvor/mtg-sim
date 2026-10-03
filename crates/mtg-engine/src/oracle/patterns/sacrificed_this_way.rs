//! "If a Saproling was sacrificed this way" (Thallid Omnivore), "If an outlaw was
//! sacrificed this way" (Boneyard Desecrator): whether a permanent the ability's cost (or
//! an earlier instruction) sacrificed matched, using the permanent as it last existed on
//! the battlefield (CR 608.2h, 400.7: the recorded object is the battlefield one).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn sacrificed_this_way(c: &str) -> Option<Condition> {
    let r = end(c).strip_suffix(" was sacrificed this way")?;
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::CountSel(Box::new(Sel::Matching(
            Box::new(Sel::Var(vars::SACRIFICED)),
            f,
        ))),
        Cmp::Ge,
        Value::c(1),
    ))
}

inventory::submit! { ConditionPattern { name: "a [object] was sacrificed this way", priority: 100, parse: sacrificed_this_way } }
