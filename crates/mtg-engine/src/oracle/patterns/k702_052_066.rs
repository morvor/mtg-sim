//! Oracle text of the keywords of CR 702.52–702.66 that the generic keyword parser
//! doesn't handle, and phrases that go with them.

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;
use crate::types::counters;

/// "it had no time counters on it" (vanishing creatures' "When ~ dies, if it had no time
/// counters on it, ..."): the permanent as it last existed on the battlefield (the dies
/// trigger's source is that object).
fn had_no_counters(l: &str) -> Option<Condition> {
    let kind = l
        .strip_prefix("it had no ")?
        .strip_suffix(" counters on it")?
        .trim();
    if kind.is_empty() || kind.contains(' ') {
        return None;
    }
    let kind = if kind == "+1/+1" { counters::PLUS1 } else { kind };
    Some(Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(kind.into())),
        Cmp::Eq,
        Value::c(0),
    ))
}

inventory::submit! { ConditionPattern { name: "it had no [kind] counters on it", priority: 100, parse: had_no_counters } }
