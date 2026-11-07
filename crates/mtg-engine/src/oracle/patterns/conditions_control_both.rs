//! Conditions on controlling two things at once: "you control an Urza's Mine and an
//! Urza's Power-Plant" (permanents with those land types, CR 205.3i), "you control
//! artifacts named Crown of Empires and Scepter of Empires" (an artifact with each name,
//! CR 201.2). Each part is checked on its own as the condition is evaluated.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

/// "a[n] [object phrase]" → its filter, when nothing follows it. (The article makes it
/// singular; the phrase parser takes the possessive in "Urza's Mine" for a plural.)
fn single(s: &str) -> Option<Filter> {
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail).is_empty().then_some(f)
}

/// "you control a[n] [X] and a[n] [Y]"
fn control_a_and_a(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    // Try each " and a"/" and an" split: the object phrases themselves may contain "and".
    let mut parts = None;
    for (i, _) in r.match_indices(" and a") {
        let (a, b) = (&r[..i], &r[i + " and ".len()..]);
        if let (Some(fa), Some(fb)) = (single(a), single(b)) {
            parts = Some((fa, fb));
            break;
        }
    }
    let (a, b) = parts?;
    Some(Condition::And(vec![
        Condition::Exists(a.you_control()),
        Condition::Exists(b.you_control()),
    ]))
}

inventory::submit! { ConditionPattern { name: "you control a [X] and a [Y]", priority: 100, parse: control_a_and_a } }

/// "you control [objects] named [A] and [B]": one of each.
fn control_named_a_and_b(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    let (kind, names) = r.split_once(" named ")?;
    let (f, plural, tail) = parse_object_phrase(kind)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    let (a, b) = names.split_once(" and ")?;
    if a.is_empty() || b.is_empty() || b.contains(" and ") {
        return None;
    }
    let named = |n: &str| {
        Condition::Exists(Filter::and(vec![f.clone(), Filter::Named(n.into())]).you_control())
    };
    Some(Condition::And(vec![named(a), named(b)]))
}

inventory::submit! { ConditionPattern { name: "you control [objects] named [A] and [B]", priority: 100, parse: control_named_a_and_b } }
