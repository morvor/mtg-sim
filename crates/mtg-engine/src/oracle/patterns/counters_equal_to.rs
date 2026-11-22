//! "Put +1/+1 counters on that creature equal to its power." (Experiment Twelve), "put a
//! number of +1/+1 counters on it equal to [amount]": that many counters (CR 122.1).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

fn counters_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let r = r.strip_prefix("a number of ").unwrap_or(r);
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.strip_prefix("counters on ")?;
    let (what_text, amount) = r.split_once(" equal to ")?;
    let saved = b.targets.len();
    let (what, rest) = object_ref(what_text, b)?;
    if !rest.trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    let (n, rest) = crate::oracle::statics::parse_value_phrase(amount, b)?;
    if !end(&rest).trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::AddCounters { what, kind, n })
}

inventory::submit! { EffectPattern { name: "put [kind] counters on [object] equal to [amount]", priority: 90, parse: counters_equal_to } }
