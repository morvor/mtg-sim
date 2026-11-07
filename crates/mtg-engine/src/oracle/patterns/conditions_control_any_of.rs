//! "you control a God, a Demigod, or a legendary enchantment" (Altar of the Pantheon): a
//! list of three or more singular object phrases, any one of which satisfies the
//! condition. (Two alternatives, "a Forest or a Plains", are parsed in `etb_choices.rs`.)

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

/// "a[n] [object phrase]" → its filter, when nothing follows it.
fn single(s: &str) -> Option<Filter> {
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail).is_empty().then_some(f)
}

/// "you control a[n] [X], a[n] [Y], or a[n] [Z]"
fn control_any_of(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    let (head, last) = r.rsplit_once(", or ")?;
    let mut parts: Vec<&str> = head.split(", ").collect();
    parts.push(last);
    if parts.len() < 3 {
        return None;
    }
    let filters = parts
        .into_iter()
        .map(|p| single(p.trim()))
        .collect::<Option<Vec<Filter>>>()?;
    Some(Condition::Exists(Filter::Or(filters).you_control()))
}

inventory::submit! { ConditionPattern { name: "you control a [X], a [Y], or a [Z]", priority: 100, parse: control_any_of } }
