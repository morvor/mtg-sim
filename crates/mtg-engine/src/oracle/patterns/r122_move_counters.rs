//! Moving counters from this permanent (CR 122.5): "Move a +1/+1 counter from ~ onto
//! target creature" (Explorer's Cache, Weapon Rack), "move two +1/+1 counters from ~ onto
//! target creature", "move all counters from ~ onto target creature". The counters are
//! removed from ~ and put on the target, so abilities that care about counters being
//! removed or put on a permanent apply; if either can't happen, nothing moves.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

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
