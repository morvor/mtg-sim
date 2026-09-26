//! Unattaching (CR 701.3d): "Unattach all Equipment from target creature."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// "unattach all equipment from target creature", "unattach all equipment from them".
fn unattach_all(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("unattach all ")?;
    let (kind, from) = r.split_once(" from ")?;
    let (f, _, tail) = parse_object_phrase(kind)?;
    if !end(tail).is_empty() {
        return None;
    }
    let (host, tail) = object_ref(from, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::ForEach {
        sel: host,
        var: vars::AFFECTED,
        effect: Box::new(Effect::Unattach {
            what: Sel::All(Filter::and(vec![
                f,
                Filter::Custom("attached_to_affected".into()),
            ])),
        }),
    })
}

inventory::submit! { EffectPattern { name: "a701 unattach all", priority: 100, parse: unattach_all } }
