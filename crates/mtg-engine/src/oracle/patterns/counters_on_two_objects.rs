//! "Put [counters] on [object] and [counters] on [another object]": one instruction that
//! puts counters on two different objects, the verb shared by both halves ("Put a +1/+1
//! counter on each creature you control and a loyalty counter on each other planeswalker
//! you control." — Ajani, the Greathearted; "put a +1/+1 counter on it and a +1/+1
//! counter on up to one other target attacking creature" — Trygon Prime). Each half is
//! carried out as its own "put" instruction, in order.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn counters_on_two_objects(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    // Try each " and " in turn: the second half starts with a count of counters and names
    // its own object ("a loyalty counter on ...").
    let mut from = 0;
    while let Some(i) = r[from..].find(" and ") {
        let at = from + i;
        from = at + 5;
        let (first, second) = (&r[..at], &r[at + 5..]);
        if !first.contains(" counter") || !first.contains(" on ") {
            continue;
        }
        let Some((counters, _)) = second.split_once(" on ") else {
            continue;
        };
        if !(counters.ends_with(" counter") || counters.ends_with(" counters"))
            || counters.contains(" and ")
        {
            continue;
        }
        let saved_targets = b.targets.len();
        let saved_it = b.it.clone();
        let Some(a) = parse_clause(&format!("put {first}"), b) else {
            b.targets.truncate(saved_targets);
            b.it = saved_it;
            continue;
        };
        let Some(c) = parse_clause(&format!("put {second}"), b) else {
            b.targets.truncate(saved_targets);
            b.it = saved_it;
            continue;
        };
        if !matches!(a, Effect::AddCounters { .. } | Effect::ForEach { .. })
            || !matches!(c, Effect::AddCounters { .. } | Effect::ForEach { .. })
        {
            b.targets.truncate(saved_targets);
            b.it = saved_it;
            continue;
        }
        return Some(Effect::Seq(vec![a, c]));
    }
    None
}

inventory::submit! { EffectPattern { name: "put counters on one object and counters on another", priority: 90, parse: counters_on_two_objects } }
