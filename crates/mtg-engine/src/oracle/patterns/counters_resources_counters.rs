//! Counters on permanents (CR 122):
//!
//! * "put a +1/+1 counter on each of up to two target creatures [you control]";
//! * "distribute two +1/+1 counters among one or two target creatures" (CR 601.2d);
//! * "double the number of +1/+1 counters on ~" / "... of each kind of counter on
//!   target permanent" (CR 701.10e);
//! * conditions on counters: "if there are three or more ki counters on ~", "if there
//!   are no depletion counters on ~", "if ~ has counters on it", "if it had a +1/+1
//!   counter on it" (the object as it last existed, CR 603.10a).

use super::{AbilityPattern, ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CounterKind;

/// The variable bound to each object whose counters are doubled.
const EACH: Var = vars::USER + 61;

/// "[n] [kind] counter(s)" → (n, kind, rest).
fn count_and_kind(s: &str) -> Option<(Value, CounterKind, &str)> {
    let (n, r) = parse_number(s)?;
    let (kind, r) = counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    Some((n, kind, r))
}

/// "put a +1/+1 counter on each of up to two target creatures [you control]", "put a
/// -1/-1 counter on each of up to two target creatures".
fn counter_on_each_of_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let (n, kind, r) = count_and_kind(r)?;
    let r = strip(r, "on each of ")?;
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || matches!(spec.max, Value::Const(1)) {
        return None;
    }
    if !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::AddCounters {
        what: Sel::Target(slot),
        kind,
        n,
    })
}

inventory::submit! { EffectPattern { name: "counters_resources: counter on each of targets", priority: 100, parse: counter_on_each_of_targets } }

/// "distribute two +1/+1 counters among one or two target creatures [you control]",
/// "... among one, two, or three target creatures", "... among up to three target
/// creatures", "... among any number of target creatures". The division is announced as
/// the spell or ability is put on the stack, and each target gets at least one counter
/// (CR 601.2d).
fn distribute_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("distribute ")?;
    let (n, kind, r) = count_and_kind(r)?;
    let r = strip(r, "among ")?;
    let total = n.as_const();
    // "any number of" and "up to N" allow zero targets (CR 115.6; "You can cast Stolen
    // Goodies with no targets"); then nothing is distributed.
    let (min, max, r): (u32, Value, &str) = if let Some(r) = strip(r, "one or two ") {
        (1, Value::c(2), r)
    } else if let Some(r) = strip(r, "one, two, or three ") {
        (1, Value::c(3), r)
    } else if let Some(r) = strip(r, "any number of ") {
        // Each target gets at least one, so there are at most that many targets.
        (0, n.clone(), r)
    } else if let Some(r) = strip(r, "up to ") {
        let (m, r2) = parse_number(r)?;
        (0, m, r2)
    } else {
        return None;
    };
    // More targets than counters would leave a target without one.
    if let (Some(t), Some(m)) = (total, max.as_const()) {
        if m > t {
            return None;
        }
    }
    if strip(r, "target ").is_none() && strip(r, "other target ").is_none() {
        return None;
    }
    let (mut spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    spec.min = min;
    spec.max = max;
    spec.divide = Some(n);
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::Custom(
        crate::counter_rules::divided_counters_effect(slot, &kind),
    ))
}

inventory::submit! { EffectPattern { name: "counters_resources: distribute counters", priority: 100, parse: distribute_counters } }

/// "double the number of +1/+1 counters on ~", "... on each creature you control",
/// "double the number of each kind of counter on target permanent": each object gets as
/// many of those counters as it already has (CR 701.10e).
fn double_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("double the number of ")?;
    let (kind, r) = if let Some(r) = r.strip_prefix("each kind of counter on ") {
        (None, r)
    } else {
        let (k, r) = counter_kind(r)?;
        (Some(k), strip(r, "counters on ")?)
    };
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::ForEach {
        sel: what,
        var: EACH,
        effect: Box::new(Effect::PutCountersOf {
            from: Sel::Var(EACH),
            to: Sel::Var(EACH),
            kind,
        }),
    })
}

inventory::submit! { EffectPattern { name: "counters_resources: double counters", priority: 100, parse: double_counters } }

/// "[kind] counter(s)" or "counters" (any kind) at the start of `s`, then " on".
fn kind_then_on(s: &str) -> Option<(Option<CounterKind>, &str)> {
    if let Some(r) = strip(s, "counters on").or_else(|| strip(s, "counter on")) {
        return Some((None, r));
    }
    let (k, r) = counter_kind(s)?;
    let r = strip(r, "counters on").or_else(|| strip(r, "counter on"))?;
    Some((Some(k), r))
}

fn counters(sel: Sel, kind: Option<CounterKind>) -> Value {
    Value::CountersOn(Box::new(sel), kind)
}

/// "N or more", "N or fewer", "no", "a"/"one or more", exactly "N" before a counter kind.
fn amount_cmp(s: &str) -> Option<(Cmp, Value, &str)> {
    if let Some(r) = strip(s, "no") {
        return Some((Cmp::Eq, Value::c(0), r));
    }
    if let Some(r) = strip(s, "one or more") {
        return Some((Cmp::Ge, Value::c(1), r));
    }
    let (n, r) = parse_number(s)?;
    if matches!(n, Value::X) {
        return None;
    }
    if let Some(r) = strip(r, "or more") {
        return Some((Cmp::Ge, n, r));
    }
    if let Some(r) = strip(r, "or fewer").or_else(|| strip(r, "or less")) {
        return Some((Cmp::Le, n, r));
    }
    if s.starts_with("a ") || s.starts_with("an ") {
        return Some((Cmp::Ge, n, r));
    }
    Some((Cmp::Eq, n, r))
}

/// Counter conditions about the source: "there are three or more ki counters on ~",
/// "there are no depletion counters on ~", "~ has two or more +1/+1 counters on it",
/// "~ has counters on it", "~ had a +1/+1 counter on it" (a source that left the
/// battlefield is looked at as it last existed, CR 603.10a).
fn counter_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    if let Some(r) = c
        .strip_prefix("there are ")
        .or_else(|| c.strip_prefix("there is "))
    {
        let (cmp, n, r) = amount_cmp(r)?;
        let (kind, r) = kind_then_on(r)?;
        if end(r) != "~" {
            return None;
        }
        return Some(Condition::Compare(counters(Sel::This, kind), cmp, n));
    }
    let r = c
        .strip_prefix("~ has ")
        .or_else(|| c.strip_prefix("~ had "))?;
    // "~ has counters on it"
    if let Some(r2) = strip(r, "counters on") {
        return matches!(end(r2), "it" | "~")
            .then(|| Condition::Compare(counters(Sel::This, None), Cmp::Ge, Value::c(1)));
    }
    let (cmp, n, r) = amount_cmp(r)?;
    let (kind, r) = kind_then_on(r)?;
    if !matches!(end(r), "it" | "~") {
        return None;
    }
    Some(Condition::Compare(counters(Sel::This, kind), cmp, n))
}

inventory::submit! { ConditionPattern { name: "counters_resources: counters on the source", priority: 100, parse: counter_condition } }

/// A triggered ability of the source about the source with an intervening "if it
/// had/has ... counter(s) on it" clause: "When ~ dies, if it had a +1/+1 counter on it,
/// draw a card." Only when the trigger's "it" is the source itself, so "it" can be read
/// as "~".
fn source_counter_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    if !(lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at ")) {
        return None;
    }
    let (cond_s, rest) = lower.split_once(", if it ")?;
    let (_, it, _) = crate::oracle::triggers::parse_trigger_condition(cond_s)?;
    if !matches!(it, Sel::This) {
        return None;
    }
    let (clause, _) = rest.split_once(", ")?;
    if !(clause.starts_with("had ") || clause.starts_with("has ")) || !clause.contains("counter") {
        return None;
    }
    // Text the standard compiler already understands ("if it had counters on it", read
    // from the object as it last existed) is left to it.
    if crate::oracle::triggers::parse_triggered(text, ctx).is_some() {
        return None;
    }
    // Rewrite the pronoun (at the same position in the original-case text).
    let i = lower.find(", if it ")?;
    if !text.is_char_boundary(i) || text.len() != lower.len() {
        return None;
    }
    let rewritten = format!("{}, if ~ {}", &text[..i], &text[i + ", if it ".len()..]);
    let a = crate::oracle::triggers::parse_triggered(&rewritten, ctx)?;
    Some(vec![AbilityDef::new(a.kind.clone(), block)])
}

inventory::submit! { AbilityPattern { name: "counters_resources: if it had counters (source trigger)", priority: 100, parse: source_counter_trigger } }
