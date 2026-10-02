//! Copying spells (CR 707.10): "Copy target instant or sorcery spell [you control]." and
//! the follow-up "You may choose new targets for the copy." (CR 707.10c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "copy target instant or sorcery spell", "copy target spell you control", "copy target
/// instant or sorcery spell twice".
fn copy_target_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy ")?;
    if !r.starts_with("target ") {
        return None;
    }
    let (r, count) = match r.strip_suffix(" twice") {
        Some(r) => (r, 2),
        None => (r, 1),
    };
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Spell(_)) {
        return None;
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::CopySpell {
        what: Sel::Target(slot),
        count: Value::c(count),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy target spell", priority: 100, parse: copy_target_spell } }

/// "copy it" / "copy that spell" in an ability that triggers on casting a spell ("Whenever
/// you cast an Adventure instant or sorcery spell, copy it."): the spell that was cast.
fn copy_trigger_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(end(l), "copy it" | "copy that spell") || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    Some(Effect::CopySpell {
        what: Sel::TriggerSpell,
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy the triggering spell", priority: 100, parse: copy_trigger_spell } }

/// "copy that spell" after a sentence about a target spell ("You may choose new targets
/// for target instant or sorcery spell. Then copy that spell."): that target spell.
fn copy_that_target_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if !matches!(l, "copy it" | "copy that spell") {
        return None;
    }
    let Sel::Target(n) = b.it else {
        return None;
    };
    if !matches!(
        b.targets.get(n as usize).map(|t| &t.what),
        Some(TargetKind::Spell(_))
    ) {
        return None;
    }
    Some(Effect::CopySpell {
        what: Sel::Target(n),
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy that target spell", priority: 100, parse: copy_that_target_spell } }

/// "You may choose new targets for the copy." after copying a spell (CR 707.10c).
fn new_targets_for_copy(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    if !matches!(
        end(&l),
        "you may choose new targets for the copy" | "you may choose new targets for the copies"
    ) {
        return false;
    }
    match copy_in(prev) {
        Some(new_targets) => {
            *new_targets = true;
            true
        }
        None => false,
    }
}

/// The `new_targets` flag of the spell copy the effect ends with: the copy itself, or an
/// optional one ("you may copy it", "you may pay {1}. If you do, copy that spell", "you
/// may remove two +1/+1 counters from ~. If you do, copy that spell").
fn copy_in(e: &mut Effect) -> Option<&mut bool> {
    match e {
        Effect::CopySpell { new_targets, .. } => Some(new_targets),
        Effect::May { effect, .. } => copy_in(effect),
        Effect::PayOptional { then, .. } => copy_in(then),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => copy_in(then),
        Effect::Seq(v) => v.last_mut().and_then(copy_in),
        _ => None,
    }
}

inventory::submit! { FollowupPattern { name: "copy: new targets", priority: 100, apply: new_targets_for_copy } }
