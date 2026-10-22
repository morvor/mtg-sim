//! Phrases of the Clockwork creatures (see `kw/combat_counter_phrases.rs`):
//!
//! * "put up to X [kind] counters on ~" (an effect), followed by "This ability can't cause
//!   the total number of [kind] counters on ~ to be greater than N." (a follow-up that caps
//!   the total);
//! * the condition "~ attacked or blocked this combat".

use super::{ConditionPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::combat_counter_phrases::{put_up_to_x, ATTACKED_OR_BLOCKED_THIS_COMBAT, PUT_UP_TO_X};
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn put_up_to_x_counters(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put up to x ")?;
    let (kind, r) = counter_kind(r)?;
    let r = strip(r, "counters")?;
    if end(r) != "on ~" {
        return None;
    }
    Some(Effect::Custom(put_up_to_x(&kind, None).into()))
}

inventory::submit! { EffectPattern { name: "r122 put up to x [kind] counters on ~", priority: 60, parse: put_up_to_x_counters } }

/// "This ability can't cause the total number of [kind] counters on ~ to be greater than
/// N." after "put up to X [kind] counters on ~".
fn cap_total_counters(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("this ability can't cause the total number of ") else {
        return false;
    };
    let Some((kind, r)) = counter_kind(r) else {
        return false;
    };
    let Some(r) = strip(r, "counters on ~ to be greater than") else {
        return false;
    };
    let Some((Value::Const(n), tail)) = parse_number(r) else {
        return false;
    };
    if !end(tail).is_empty() || n < 0 {
        return false;
    }
    let Effect::Custom(name) = prev else {
        return false;
    };
    if name.as_str() != put_up_to_x(&kind, None) || !name.starts_with(PUT_UP_TO_X) {
        return false;
    }
    *name = put_up_to_x(&kind, Some(n as u32)).into();
    true
}

inventory::submit! { FollowupPattern { name: "r122 this ability can't cause the total number of counters to be greater than N", priority: 60, apply: cap_total_counters } }

fn attacked_or_blocked_this_combat(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "~ attacked or blocked this combat" | "it attacked or blocked this combat"
    )
    .then(|| Condition::Custom(ATTACKED_OR_BLOCKED_THIS_COMBAT.into()))
}

inventory::submit! { ConditionPattern { name: "r122 ~ attacked or blocked this combat", priority: 60, parse: attacked_or_blocked_this_combat } }
