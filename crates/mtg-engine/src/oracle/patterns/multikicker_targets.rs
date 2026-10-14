//! Multikicker (CR 702.33c) spells whose number of targets depends on how many times they
//! were kicked: "Choose any target, then choose another target for each time this spell
//! was kicked." (Comet Storm), "Choose target creature, then choose another target
//! creature for each time this spell was kicked." (Strength of the Tajuru).
//!
//! The number of times a spell is kicked is announced before its targets are chosen
//! (CR 601.2b, 601.2c), so the spell has one target plus one for each time it was kicked,
//! all in one target slot: each of them must be different (CR 115.3). The sentence that
//! follows refers to them as "each of them".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_any_target};

fn choose_targets_for_each_kick(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose ")?;
    let (first, rest) = r.split_once(", then choose another ")?;
    let more = rest
        .strip_suffix(" for each time this spell was kicked")
        .or_else(|| rest.strip_suffix(" for each time ~ was kicked"))?;
    // "another target (creature)" names the same kind of target as the first one.
    if first != more && !(first == "any target" && more == "target") {
        return None;
    }
    let (mut spec, tail) = parse_any_target(first)?;
    if !end(tail).is_empty() || spec.fixed_min() != Some(1) || !matches!(spec.max, Value::Const(1)) {
        return None;
    }
    // Exactly one more than the number of times it was kicked (CR 601.2c).
    let n = Value::Sum(vec![Value::c(1), Value::TimesKicked]);
    spec.min = n.clone();
    spec.max = n;
    let slot = b.add_target(spec, first);
    b.it = Sel::Target(slot);
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "multikicker: choose a target, then another for each time it was kicked", priority: 90, parse: choose_targets_for_each_kick } }
