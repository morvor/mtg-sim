//! Setting power or toughness from a creature it's in combat with (layer 7b, CR 613.4b),
//! the values determined as the ability resolves (CR 608.2h):
//!
//! * "~'s base toughness becomes equal to 1 plus the power of target creature blocking or
//!   blocked by ~" (Sentinel; the effect lasts indefinitely);
//! * "~'s power becomes the toughness of target creature blocking or being blocked by ~
//!   minus 1 until end of turn, and its toughness becomes 1 plus the power of that creature
//!   until end of turn" (Sworn Defender).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Parses a target phrase ("target creature blocking or [being] blocked by ~") that must
/// be the whole of `s`, adding the target. Returns its slot.
fn whole_target(s: &str, b: &mut Builder) -> Option<u8> {
    let s = s.replace("blocking or being blocked by", "blocking or blocked by");
    let (spec, tail) = parse_target(&s)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(b.add_target(spec, &s))
}

fn plus(n: Value, v: Value) -> Value {
    Value::Sum(vec![n, v])
}

/// "~'s base toughness becomes equal to N plus the power of [target]" (and the same with
/// base power / toughness of).
fn base_stat_from_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("~'s base ")?;
    let (toughness, r) = match r.strip_prefix("toughness becomes equal to ") {
        Some(r) => (true, r),
        None => (false, r.strip_prefix("power becomes equal to ")?),
    };
    let (n, r) = parse_number(r)?;
    let (of_power, r) = match strip(r, "plus the power of") {
        Some(r) => (true, r),
        None => (false, strip(r, "plus the toughness of")?),
    };
    let slot = whole_target(r, b)?;
    let target = Box::new(Sel::Target(slot));
    let v = plus(
        n,
        if of_power {
            Value::PowerOf(target)
        } else {
            Value::ToughnessOf(target)
        },
    );
    let set = if toughness {
        Modification::SetPT(None, Some(v))
    } else {
        Modification::SetPT(Some(v), None)
    };
    Some(Effect::Modify {
        what: Sel::This,
        mods: vec![set],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "r613 ~'s base toughness becomes N plus the power of target", priority: 60, parse: base_stat_from_target } }

/// "~'s power becomes the toughness of [target] minus 1 until end of turn, and its
/// toughness becomes 1 plus the power of that creature until end of turn".
fn swap_stats_with_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("~'s power becomes the toughness of ")?;
    let target = r.strip_suffix(
        " minus 1 until end of turn, and its toughness becomes 1 plus the power of that creature until end of turn",
    )?;
    let slot = whole_target(target, b)?;
    let t = Box::new(Sel::Target(slot));
    Some(Effect::Modify {
        what: Sel::This,
        mods: vec![Modification::SetPT(
            Some(Value::Diff(Box::new(Value::ToughnessOf(t.clone())), Box::new(Value::c(1)))),
            Some(plus(Value::c(1), Value::PowerOf(t))),
        )],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "r613 ~'s power becomes the toughness of target minus 1, toughness 1 plus its power", priority: 60, parse: swap_stats_with_target } }
