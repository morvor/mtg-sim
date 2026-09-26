//! Copying spells (CR 707.10): "Copy target instant or sorcery spell [you control]." and
//! the follow-up "You may choose new targets for the copy." (CR 707.10c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "copy target instant or sorcery spell", "copy target spell you control".
fn copy_target_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy ")?;
    if !r.starts_with("target ") {
        return None;
    }
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Spell(_)) {
        return None;
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::CopySpell {
        what: Sel::Target(slot),
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy target spell", priority: 100, parse: copy_target_spell } }

/// "You may choose new targets for the copy." after copying a spell (CR 707.10c).
fn new_targets_for_copy(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if !matches!(
        end(&l),
        "you may choose new targets for the copy" | "you may choose new targets for the copies"
    ) {
        return false;
    }
    match prev {
        Effect::CopySpell { new_targets, .. } => {
            *new_targets = true;
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "copy: new targets", priority: 100, apply: new_targets_for_copy } }
