//! "Choose a basic land type. Each land you control becomes that type until end of turn."
//! (Elsewhere Flask, Terraformer): "that type" is the basic land type chosen by the
//! previous sentence. The lands get that land type in place of their other land types and
//! lose the abilities from their rules text (CR 305.7); the set of lands is locked in as
//! the effect begins (CR 611.2c).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase, parse_target};

/// Whether the effect ends with choosing a basic land type.
fn ends_with_land_type_choice(e: &Effect) -> bool {
    match e {
        Effect::Choose {
            kind: ChoiceKind::BasicLandType,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_land_type_choice),
        _ => false,
    }
}

fn becomes_chosen_land_type(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_land_type_choice(prev) {
        return false;
    }
    let l = end(l);
    let (l, duration) = match l.strip_suffix(" until end of turn") {
        Some(x) => (x, Duration::EndOfTurn),
        None => (l, Duration::Permanent),
    };
    let Some(subject) = l
        .strip_suffix(" becomes that type")
        .or_else(|| l.strip_suffix(" become that type"))
    else {
        return false;
    };
    let what = if let Some(group) = subject.strip_prefix("each ") {
        let Some((f, plural, tail)) = parse_object_phrase(group) else {
            return false;
        };
        if plural || !end(tail).is_empty() {
            return false;
        }
        Sel::All(f)
    } else if subject.starts_with("target ") {
        let Some((spec, tail)) = parse_target(subject) else {
            return false;
        };
        if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
            return false;
        }
        Sel::Target(b.add_target(spec, subject))
    } else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Modify {
            what,
            mods: vec![Modification::SetChosenBasicLandType],
            duration,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "[lands] become that (chosen basic land) type", priority: 100, apply: becomes_chosen_land_type } }
