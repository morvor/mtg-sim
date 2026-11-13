//! Attaching several permanents (CR 701.3): "For each Aura and Equipment you control, you
//! may attach it to a creature you control." (Inventory Management). See
//! `kw/attach_each.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::attach_each::attach_each;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// An object phrase naming a whole group: "equipment you control", or two nouns sharing
/// what follows the second ("aura and equipment you control").
fn group(s: &str) -> Option<Filter> {
    if let Some((f, _, tail)) = parse_object_phrase(s) {
        if end(tail).is_empty() {
            return Some(f);
        }
    }
    let (a, b) = s.split_once(" and ")?;
    if a.contains(' ') {
        return None;
    }
    let (fb, _, tail) = parse_object_phrase(b)?;
    if !end(tail).is_empty() {
        return None;
    }
    let shared = b.split_once(' ').map_or("", |(_, r)| r);
    let a = format!("{a} {shared}");
    let (fa, _, tail) = parse_object_phrase(a.trim())?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Filter::Or(vec![fa, fb]))
}

/// "for each [group], [you may] attach it to a [object]".
fn attach_each_to_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (what, rest) = r.split_once(", ")?;
    let (optional, rest) = match rest.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let to = rest.strip_prefix("attach it to ")?;
    let to = to.strip_prefix("a ").or_else(|| to.strip_prefix("an "))?;
    let (to, plural, tail) = parse_object_phrase(to)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let _ = b;
    Some(attach_each(group(what)?, to, optional))
}

inventory::submit! { EffectPattern { name: "a701 for each, attach it to a chosen object", priority: 100, parse: attach_each_to_chosen } }
