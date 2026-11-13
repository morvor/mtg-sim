//! "Turn target face-down creature [an opponent controls / you control] face up." (Break
//! Open, Ixidor, Reality Sculptor, Expose the Culprit): an effect turning the targeted
//! face-down permanent face up without paying a cost (CR 708.8). A manifested or cloaked
//! instant or sorcery card is revealed instead and stays face down (CR 701.40g, 701.58g),
//! and one an effect says can't be turned face up stays face down (CR 708.7).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_target};

fn turn_target_face_up(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("turn ")?;
    if !r.starts_with("target ") {
        return None;
    }
    let (spec, tail) = parse_target(r)?;
    if end(tail).trim() != "face up" {
        return None;
    }
    fn has_face_down(f: &Filter) -> bool {
        matches!(f, Filter::FaceDown) || matches!(f, Filter::And(v) if v.iter().any(has_face_down))
    }
    match &spec.what {
        TargetKind::Object(f) if has_face_down(f) => {}
        _ => return None,
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::TurnFaceUp {
        what: Sel::Target(slot),
    })
}

inventory::submit! { EffectPattern { name: "r708 turn target face-down creature face up", priority: 0, parse: turn_target_face_up } }
