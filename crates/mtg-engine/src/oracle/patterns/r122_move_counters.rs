//! Moving counters (CR 122.5): "Move a +1/+1 counter from ~ onto target creature."
//! (Simic Fluxmage, Weapon Rack), "Move a +1/+1 counter from target creature onto a second
//! target creature." (Daghatar the Adamant), "Move a counter from target creature onto a
//! second target creature." (Leech Bonder: a counter of a kind the player chooses), "move
//! all counters from ~ onto target creature" (The Ozolith), "move any number of +1/+1
//! counters from ~ onto another target creature" (Scrounging Bandar: the number is chosen
//! as it resolves), "move X +1/+1 counters ...", "move a +1/+1 counter from target
//! creature onto another target creature with the same controller" (Simic Guildmage,
//! Bioshift: the two targets must have the same controller as they're chosen and as the
//! ability resolves, `TargetSpec::related_to`). The counters are removed from the first
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
    // "that creature": the trigger's object.
    if s == "that creature" && matches!(b.it, Sel::TriggerObject) {
        return Some(Sel::TriggerObject);
    }
    // "another target creature with the same controller": a different object than the
    // earlier target, controlled by the same player.
    let (same_controller, s) = match s.strip_suffix(" with the same controller") {
        Some(r) if r.starts_with("another target ") && !b.targets.is_empty() => (true, r),
        Some(_) => return None,
        None => (false, s),
    };
    // "a second target creature": a different object than the earlier targets (which may
    // be the source, unlike "another target creature").
    let (second, text) = match s.strip_prefix("a second target ") {
        Some(r) => (true, format!("target {r}")),
        None => (false, s.to_string()),
    };
    let (mut spec, tail) = parse_target(&text)?;
    if !end(tail).is_empty()
        || spec.fixed_min() != Some(1)
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
    if same_controller {
        let first = b.targets.len() as u8 - 1;
        spec.distinct_from = vec![first];
        spec.related_to = Some((first, TargetGroup::SameController));
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
    if let Some(e) = move_group(from, onto, &kind, &n, any_number, b) {
        return Some(e);
    }
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

/// The variable bound to each object of a group counters move from or onto.
const EACH: Var = vars::USER + 4124;

/// Moves between one object and each object of a group (CR 122.5): "move all +1/+1
/// counters from all creatures onto it" (Spike Cannibal), "move any number of +1/+1
/// counters from other permanents you control onto ~" (Aetherborn Marauder), "move any
/// number of +1/+1 counters from ~ onto other creatures" (Forgotten Ancient). For each
/// object of the group in turn, the counters (all, or as many as the player chooses) are
/// moved.
fn move_group(
    from: &str,
    onto: &str,
    kind: &Option<crate::types::CounterKind>,
    n: &Option<Value>,
    any_number: bool,
    b: &mut Builder,
) -> Option<Effect> {
    let group = |s: &str, b: &mut Builder| -> Option<Filter> {
        let s = end(s);
        let s = s.strip_prefix("all ").unwrap_or(s);
        let (f, plural, tail) = parse_object_phrase(s)?;
        let _ = b;
        (plural && end(tail).is_empty()).then(|| Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]))
    };
    let single = |s: &str, b: &mut Builder| -> Option<Sel> {
        match end(s) {
            "~" => Some(Sel::This),
            "it" if matches!(b.it, Sel::This) => Some(Sel::This),
            _ => None,
        }
    };
    // Only "all" or "any number of" counters move between an object and a group.
    if !(n.is_none() || any_number) {
        return None;
    }
    let (filter, from_group) = if let Some(f) = group(from, b) {
        single(onto, b)?;
        (f, true)
    } else {
        single(from, b)?;
        (group(onto, b)?, false)
    };
    let one = if from_group {
        single(onto, b)?
    } else {
        single(from, b)?
    };
    let (from_sel, to_sel) = if from_group {
        (Sel::Var(EACH), one)
    } else {
        (one, Sel::Var(EACH))
    };
    let mv = Effect::MoveCounters {
        from: from_sel,
        to: to_sel,
        kind: kind.clone(),
        n: n.clone(),
    };
    let body = if any_number {
        Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Number { min: 0, max: 1000 },
            },
            mv,
        ])
    } else {
        mv
    };
    Some(Effect::ForEach {
        sel: Sel::All(filter),
        var: EACH,
        effect: Box::new(body),
    })
}
