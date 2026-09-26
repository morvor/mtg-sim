//! Transforming (CR 701.27):
//!
//! * "Whenever a permanent you control transforms into a non-Human creature", "... into a
//!   Phyrexian": the permanent has the quality immediately after it transforms
//!   (CR 701.27e). ("~ transforms into ~" is parsed with the other status triggers.)

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn transforms_into(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (subj, quality) = r.split_once(" transforms into ")?;
    if quality.contains('~') {
        return None;
    }
    let subject = if subj == "~" {
        Filter::Source
    } else {
        let x = subj
            .strip_prefix("a ")
            .or_else(|| subj.strip_prefix("an "))?;
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    let q = quality
        .strip_prefix("a ")
        .or_else(|| quality.strip_prefix("an "))?;
    let (qf, _, tail) = parse_object_phrase(q)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::Transforms(Filter::and(vec![subject, qf])),
        Sel::TriggerObject,
        PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
    ))
}

inventory::submit! { TriggerPattern { name: "a701 transforms into a quality", priority: 100, parse: transforms_into } }
