//! "there are thirty or more counters among artifacts and creatures you control" (Lux
//! Artillery), "there are four or more lore counters among Sagas you control": the total
//! number of counters (of that kind) on those permanents, checked when the condition is
//! (CR 603.4 for an intervening "if").

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

fn counters_among(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("there are ")?;
    let (n, rest) = parse_number(r)?;
    let rest = rest.trim_start().strip_prefix("or more ")?;
    let (kind, objs) = if let Some(o) = rest.strip_prefix("counters among ") {
        (None, o)
    } else {
        let (kind, r2) = crate::oracle::costs::counter_kind(rest)?;
        if matches!(kind.as_str(), "different" | "kinds") {
            return None;
        }
        (Some(kind), r2.trim_start().strip_prefix("counters among ")?)
    };
    // "artifacts and creatures you control": each permanent that's either.
    let either = objs.replacen(" and ", " and/or ", 1);
    let (f, true, tail) = parse_object_phrase(objs)
        .filter(|(_, _, t)| end(t).is_empty())
        .or_else(|| parse_object_phrase(&either))?
    else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let total = Value::Aggregate(AggOp::Sum, Stat::Counters(kind), Box::new(Sel::All(f)));
    Some(Condition::Compare(total, Cmp::Ge, n))
}

inventory::submit! { ConditionPattern { name: "there are N or more counters among [objects]", priority: 100, parse: counters_among } }
