//! "Put [counters] and [counters] on [object]": counters of several kinds put on the same
//! object by one instruction ("Put a +1/+1 counter and an indestructible counter on ~.",
//! "Put two +1/+1 counters and a flying counter on target creature.", "put a +1/+1
//! counter, a flying counter, and a lifelink counter on it"). Each kind is put on it in
//! turn, as separate instructions would.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn several_kinds(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let (list, object) = r.rsplit_once(" on ")?;
    if !list.contains(" and ") {
        return None;
    }
    let items: Vec<&str> = list
        .split(", and ")
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" and "))
        .map(str::trim)
        .collect();
    if items.len() < 2
        || items
            .iter()
            .any(|i| !(i.ends_with(" counter") || i.ends_with(" counters")))
    {
        return None;
    }
    // The first kind names the object (and adds its target, if any) ...
    let first = parse_clause(&format!("put {} on {object}", items[0]), b)?;
    let Effect::AddCounters { what, .. } = &first else {
        return None;
    };
    let what = what.clone();
    let mut out = vec![first];
    // ... and the others go on the same object.
    for item in &items[1..] {
        let targets = b.targets.len();
        let e = parse_clause(&format!("put {item} on ~"), b)?;
        if b.targets.len() != targets {
            return None;
        }
        let Effect::AddCounters { kind, n, .. } = e else {
            return None;
        };
        out.push(Effect::AddCounters {
            what: what.clone(),
            kind,
            n,
        });
    }
    Some(Effect::Seq(out))
}

inventory::submit! { EffectPattern { name: "put counters of several kinds on an object", priority: 300, parse: several_kinds } }
