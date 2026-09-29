//! Moving counters (CR 122.5): "Move a +1/+1 counter from ~ onto target creature."
//! (Simic Fluxmage, Weapon Rack), "Move a +1/+1 counter from target creature onto a second
//! target creature." (Daghatar the Adamant), "Move a counter from target creature onto a
//! second target creature." (Leech Bonder: a counter of a kind the player chooses), "move
//! all counters from ~ onto target creature" (The Ozolith), "move any number of +1/+1
//! counters from ~ onto another target creature" (Scrounging Bandar: the number is chosen
//! as it resolves), "move X +1/+1 counters ...". The counters are removed from the first
//! object and put on the second, so abilities that care about counters being removed or
//! put on apply; if either can't happen, none is moved (`counter_rules::move_counters`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "~", "it" (the source or the trigger's object), or a target ("target creature",
/// "a second target creature", "another target creature").
fn object(s: &str, b: &mut Builder) -> Option<Sel> {
    let s = end(s);
    if s == "~" {
        return Some(Sel::This);
    }
    if s == "it" && matches!(b.it, Sel::This | Sel::TriggerObject) {
        return Some(b.it.clone());
    }
    // "a second target creature": a different object than the earlier targets (which may
    // be the source, unlike "another target creature").
    let (second, text) = match s.strip_prefix("a second target ") {
        Some(r) => (true, format!("target {r}")),
        None => (false, s.to_string()),
    };
    let (mut spec, tail) = parse_target(&text)?;
    if !end(tail).is_empty()
        || spec.min != 1
        || !matches!(spec.max, Value::Const(1))
        || !matches!(spec.what, TargetKind::Object(_))
    {
        return None;
    }
    if second {
        spec.distinct_from = (0..b.targets.len() as u8).collect();
        if spec.distinct_from.is_empty() {
            return None;
        }
    }
    let slot = b.add_target(spec, s);
    Some(Sel::Target(slot))
}

fn move_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("move ")?;
    // How many: "all", "any number of" (chosen as it resolves), "a", "two", "x".
    let mut any_number = false;
    let (n, r) = if let Some(r) = r.strip_prefix("all ") {
        (None, r)
    } else if let Some(r) = r.strip_prefix("any number of ") {
        any_number = true;
        (Some(Value::Chosen), r)
    } else {
        let (n, r) = parse_number(r)?;
        (Some(n), r)
    };
    // Of which kind: "+1/+1 counters", or "counters" of any kind.
    let (kind, r) = match strip(r, "counters").or_else(|| strip(r, "counter")) {
        Some(r) => (None, r),
        None => {
            let (k, r) = counter_kind(r)?;
            (Some(k), strip(r, "counters").or_else(|| strip(r, "counter"))?)
        }
    };
    let r = r.trim_start().strip_prefix("from ")?;
    let (from, onto) = r.split_once(" onto ")?;
    let saved = (b.targets.len(), b.it.clone());
    let parsed = (|| {
        let from = object(from, b)?;
        let to = object(onto, b)?;
        Some((from, to))
    })();
    let Some((from, to)) = parsed else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    let mv = Effect::MoveCounters { from, to, kind, n };
    if !any_number {
        return Some(mv);
    }
    // "Any number of": the player chooses how many as the ability resolves (more than
    // there are moves them all).
    Some(Effect::seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Number { min: 0, max: 1000 },
        },
        mv,
    ]))
}

inventory::submit! { EffectPattern { name: "move N [kind] counters from [object] onto [object]", priority: 100, parse: move_counters } }

/// "Move a +1/+1 counter from ~ onto target creature" (Explorer's Cache, Weapon Rack),
/// "move two +1/+1 counters from ~ onto target creature", "move all counters from ~ onto
/// target creature", with any target phrase. Tried first (lower priority); the pattern
/// above handles moves between targets, from "it", and "any number of" counters.
fn move_counters_from_this(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("move ")?;
    let (n, r) = match strip(r, "all") {
        Some(r) => (None, r),
        None => {
            let (n, r) = parse_number(r)?;
            (Some(n), r)
        }
    };
    // "a counter" (of any kind) or "a +1/+1 counter".
    let (kind, r) = match strip(r, "counters").or_else(|| strip(r, "counter")) {
        Some(r) => (None, r),
        None => {
            let (k, r) = counter_kind(r)?;
            (Some(k), strip(r, "counters").or_else(|| strip(r, "counter"))?)
        }
    };
    let r = strip(r, "from ~ onto")?;
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let slot = b.add_target(spec, r);
    Some(Effect::MoveCounters {
        from: Sel::This,
        to: Sel::Target(slot),
        kind,
        n,
    })
}

inventory::submit! { EffectPattern { name: "r122 move counters from ~ onto target", priority: 60, parse: move_counters_from_this } }
