//! Oracle pattern for putting a counter of a kind the player chooses on an object (CR
//! 122.1): "put your choice of a reach, menace, trample, or haste counter on target
//! Dinosaur" (Owen Grady, Raptor Trainer), "put your choice of a reach counter, a
//! vigilance counter, or a trample counter on it", "put your choice of a +1/+1 counter or
//! a first strike counter on [object]". The choice is made as the instruction is carried
//! out.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::types::CounterKind;

/// A counter kind named on its own: "+1/+1", "reach", "first strike".
fn counter_name(s: &str) -> Option<CounterKind> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if crate::layers::keyword_counter(s).is_some() {
        return Some(s.into());
    }
    let (k, rest) = crate::oracle::costs::counter_kind(s)?;
    rest.trim().is_empty().then_some(k)
}

/// "a reach, menace, trample, or haste counter", "a reach counter, a vigilance counter,
/// or a trample counter", "a +1/+1 counter or a flying counter": the kinds, at least two.
fn counter_kinds(s: &str) -> Option<Vec<CounterKind>> {
    let mut out = Vec::new();
    for item in s
        .split(", or ")
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
    {
        let item = item.trim();
        let item = item
            .strip_prefix("a ")
            .or_else(|| item.strip_prefix("an "))
            .unwrap_or(item);
        let item = item.strip_suffix(" counter").unwrap_or(item);
        out.push(counter_name(item)?);
    }
    // Only the first item needs "a"; only the last (or each) needs "counter".
    if !(s.starts_with("a ") || s.starts_with("an ")) || !s.ends_with(" counter") {
        return None;
    }
    (out.len() >= 2).then_some(out)
}

/// "put your choice of [a kind, a kind, or a kind] counter on [object]".
fn put_counter_of_your_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put your choice of ")?;
    let (list, obj) = r.split_once(" counter on ")?;
    let kinds = counter_kinds(&format!("{list} counter"))?;
    let (what, tail) = object_ref(obj, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let options = kinds
        .into_iter()
        .map(|kind| {
            (
                format!("{kind} counter"),
                Effect::AddCounters {
                    what: what.clone(),
                    kind,
                    n: Value::c(1),
                },
            )
        })
        .collect();
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options,
    })
}

inventory::submit! { EffectPattern { name: "put your choice of a counter", priority: 80, parse: put_counter_of_your_choice } }
