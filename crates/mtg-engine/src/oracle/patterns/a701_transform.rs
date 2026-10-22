//! Transforming (CR 701.27):
//!
//! * "Whenever a permanent you control transforms into a non-Human creature", "... into a
//!   Phyrexian": the permanent has the quality immediately after it transforms
//!   (CR 701.27e). ("~ transforms into ~" is parsed with the other status triggers.)
//! * "Whenever a permanent you control enters transformed": it enters back face up.
//! * "transform it", "transform him": transform the source or the object the ability
//!   triggered on, when an earlier part of the ability refers to it ("Whenever ~ attacks and isn't blocked, you
//!   may pay {2}{B}. If you do, transform it."), which then transforms only if it hasn't
//!   transformed since the ability was put onto the stack (CR 701.27f).

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

fn transform_pronoun(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("transform ")?;
    if !["it", "him", "her", "that creature", "that permanent"].contains(&r) {
        return None;
    }
    let (what, rest) = object_ref(r, b)?;
    // Only the source or the object an ability triggered on: a pronoun that the parser
    // resolves to something else (a card discarded earlier in "If a creature card is
    // discarded this way, untap ~, then transform it.") may not be what it means.
    if !end(&rest).is_empty() || !matches!(what, Sel::This | Sel::TriggerObject) {
        return None;
    }
    Some(Effect::Transform { what })
}

inventory::submit! { EffectPattern { name: "a701 transform it", priority: 100, parse: transform_pronoun } }

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

/// "Whenever a permanent you control enters transformed" (Corruption of Towashi): it
/// enters with its back face up, a transformed permanent (CR 701.27g, 712.14a).
fn enters_transformed(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let subj = end(r).strip_suffix(" enters transformed")?;
    let x = subj
        .strip_prefix("a ")
        .or_else(|| subj.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(x)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::EntersBattlefield(Filter::and(vec![
            f,
            Filter::Custom(crate::transform_rules::TRANSFORMED.into()),
        ])),
        Sel::TriggerObject,
        PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
    ))
}

inventory::submit! { TriggerPattern { name: "a701 enters transformed", priority: 100, parse: enters_transformed } }
