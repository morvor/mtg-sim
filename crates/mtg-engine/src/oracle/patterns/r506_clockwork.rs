//! The Clockwork creatures (Clockwork Avian, Clockwork Beast, Clockwork Steed, Clockwork
//! Swarm):
//!
//! * "At end of combat, if ~ attacked or blocked this combat, remove a +1/+0 counter from
//!   it." — "attacked or blocked this combat": it was an attacking or blocking creature
//!   during this combat phase (CR 506.2, 508.1, 509.1), even if it dealt no damage or was
//!   removed from combat;
//! * "{X}, {T}: Put up to X +1/+0 counters on ~. This ability can't cause the total number
//!   of +1/+0 counters on ~ to be greater than four." — the limit is checked as the
//!   ability resolves: counters over it simply aren't put on. (Putting fewer than X is the
//!   same as having chosen a smaller X.)

use super::{ConditionPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, strip};

fn attacked_or_blocked(c: &str) -> Option<Condition> {
    (end(c) == "~ attacked or blocked this combat").then(|| {
        Condition::SelMatches(
            Sel::This,
            Filter::Custom(crate::custom::ATTACKED_OR_BLOCKED_THIS_COMBAT.into()),
        )
    })
}

/// "put up to X [kind] counters on ~".
fn put_up_to_x(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put up to ")?;
    let (n, r) = parse_number(r)?;
    if !matches!(n, Value::X) {
        return None;
    }
    let (kind, r) = counter_kind(r)?;
    let r = strip(r, "counters")?;
    if end(r) != "on ~" {
        return None;
    }
    Some(Effect::AddCounters {
        what: Sel::This,
        kind,
        n,
    })
}

/// "This ability can't cause the total number of [kind] counters on ~ to be greater than
/// N." after putting counters on ~.
fn counter_total_limit(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("this ability can't cause the total number of ") else {
        return false;
    };
    let Some((kind, r)) = counter_kind(r) else {
        return false;
    };
    let Some(r) = strip(r, "counters on ~ to be greater than ") else {
        return false;
    };
    let Some((max, tail)) = parse_number(r) else {
        return false;
    };
    if !end(tail).is_empty() || matches!(max, Value::X) {
        return false;
    }
    let Effect::AddCounters {
        what: Sel::This,
        kind: k,
        n,
    } = prev
    else {
        return false;
    };
    if *k != kind {
        return false;
    }
    let room = Value::Max(
        Box::new(Value::Diff(
            Box::new(max),
            Box::new(Value::CountersOn(Box::new(Sel::This), Some(kind))),
        )),
        Box::new(Value::c(0)),
    );
    *n = Value::Min(Box::new(n.clone()), Box::new(room));
    true
}

inventory::submit! { ConditionPattern { name: "~ attacked or blocked this combat", priority: 100, parse: attacked_or_blocked } }
inventory::submit! { EffectPattern { name: "put up to X [kind] counters on ~", priority: 100, parse: put_up_to_x } }
inventory::submit! { FollowupPattern { name: "this ability can't cause the total number of [kind] counters on ~ to be greater than N", priority: 100, apply: counter_total_limit } }
