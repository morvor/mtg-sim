//! Oracle patterns for counters (CR 122): counter limits (CR 122.4), "when the Nth
//! counter is put on" triggers (CR 122.7), and putting an object's counters on another
//! object (CR 122.8, 122.9).

use super::{ConditionPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_target, split_word, strip};
use crate::oracle::CompileContext;

/// "first", "second", ..., "tenth".
fn ordinal(w: &str) -> Option<u32> {
    Some(match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        "tenth" => 10,
        _ => return None,
    })
}

/// "~ can't have more than N [kind] counters on it" (CR 122.4).
fn counter_limit(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("~ can't have more than ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?.max(0) as u32;
    let (kind, r) = counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    if end(r) != "on it" {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Custom(
        crate::counter_rules::counter_limit_name(&kind, n),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// "the Nth [kind] counter is put on ~" (CR 122.7).
fn nth_counter(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r).strip_prefix("the ")?;
    let (w, r) = split_word(r);
    let n = ordinal(w)?;
    let (kind, r) = counter_kind(r)?;
    if r != "counter is put on ~" {
        return None;
    }
    Some((
        TriggerCond::CounterThreshold {
            filter: Filter::Source,
            kind,
            n,
        },
        Sel::This,
        PlayerRef::You,
    ))
}

/// "put its counters on [target]", "put those counters on ~", "put its +1/+1 counters
/// onto up to one target creature": the same number of each kind of counter the object
/// (which has left the battlefield) had (CR 122.8, 122.9).
fn put_its_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("put ")?;
    let r = r
        .strip_prefix("its ")
        .or_else(|| r.strip_prefix("those "))
        .or_else(|| r.strip_prefix("their "))?;
    let (kind, r) = match strip(r, "counters") {
        Some(r) => (None, r),
        None => {
            let (k, r) = counter_kind(r)?;
            (Some(k), strip(r, "counters")?)
        }
    };
    let r = strip(r, "onto").or_else(|| strip(r, "on"))?;
    let to = if end(r) == "~" {
        Sel::This
    } else {
        let (spec, tail) = parse_target(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        let text = spec.text.clone();
        let from = b.it.clone();
        let slot = b.add_target(spec, &text);
        b.it = from;
        Sel::Target(slot)
    };
    let from = if b.in_trigger {
        Sel::TriggerLki
    } else {
        Sel::This
    };
    Some(Effect::PutCountersOf { from, to, kind })
}

/// "it had counters on it" (an object that left the battlefield, CR 122.8).
fn had_counters(c: &str) -> Option<Condition> {
    matches!(
        c,
        "it had counters on it" | "it had one or more counters on it"
    )
    .then(|| Condition::SelMatches(Sel::TriggerLki, Filter::HasCounter(None)))
}

inventory::submit! { StaticPattern { name: "counter limit", priority: 0, parse: counter_limit } }
inventory::submit! { ConditionPattern { name: "had counters", priority: 0, parse: had_counters } }
inventory::submit! { TriggerPattern { name: "nth counter put", priority: 0, parse: nth_counter } }
inventory::submit! { EffectPattern { name: "put its counters on", priority: 0, parse: put_its_counters } }
