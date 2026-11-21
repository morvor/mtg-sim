//! Spell targets VI (CR 115.7, 115.9, 707.10).
//!
//! * "Gain control of target spell that targets only a single permanent or player."
//!   (Chef's Kiss): a gain-control target with a qualifier about the spell's own targets
//!   (CR 115.9c).
//! * "Reselect the targets at random for the spell and the copy" after "Copy it, then"
//!   (Chef's Kiss), "reselect its target at random" (Grip of Chaos's wording): each target
//!   chosen again at random among the legal ones ([`Effect::ReselectTargetsAtRandom`],
//!   CR 115.7d, 707.10c).
//! * "The new targets can't be you or a permanent you control.": the targets that
//!   reselection can't choose.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "gain control of target spell that targets only a single permanent or player": the
/// stack target with its qualifier (without one, the general gain-control phrase).
fn gain_control_of_qualified_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("gain control of ")?;
    if super::r115_targets::split_targets_qualifier(r).1.is_none() {
        return None;
    }
    let saved = b.targets.len();
    let Some(what) = super::r115_targets::stack_target(r, b) else {
        b.targets.truncate(saved);
        return None;
    };
    b.it = what.clone();
    Some(Effect::GainControl {
        what,
        who: PlayerRef::You,
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "spell targets 6: gain control of target spell that targets only ...", priority: 90, parse: gain_control_of_qualified_spell } }

/// Whether the ability copied a spell before this sentence ("copy it, then ..."), so that
/// "the copy" is the copy it created (`vars::CREATED`).
fn copied_before() -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    raw.contains("copy it, then reselect") || raw.contains("copy it. reselect")
}

/// "the spell and the copy", "it", "its": the stack objects whose targets are reselected.
fn reselected(s: &str, b: &Builder) -> Option<Sel> {
    let spell = match &b.it {
        sel @ (Sel::Target(_) | Sel::TriggerSpell) => sel.clone(),
        _ => return None,
    };
    match s {
        "the spell and the copy" if copied_before() => {
            Some(Sel::Union(vec![spell, Sel::Var(vars::CREATED)]))
        }
        "the spell" | "it" | "that spell" => Some(spell),
        _ => None,
    }
}

/// "reselect the targets at random for X", "reselect its target(s) at random".
fn reselect_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let what = if let Some(r) = l
        .strip_prefix("reselect the targets at random for ")
        .or_else(|| l.strip_prefix("reselect the target at random for "))
    {
        reselected(r, b)?
    } else if matches!(
        l,
        "reselect its target at random" | "reselect its targets at random"
    ) {
        reselected("it", b)?
    } else {
        return None;
    };
    Some(Effect::ReselectTargetsAtRandom { what, except: None })
}

inventory::submit! { EffectPattern { name: "spell targets 6: reselect the targets at random", priority: 90, parse: reselect_at_random } }

/// One thing a new target can't be: "you", "a permanent you control".
fn excluded(s: &str) -> Option<Sel> {
    match s {
        "you" => return Some(Sel::Players(PlayerRef::You)),
        "an opponent" => return Some(Sel::Players(PlayerRef::EachOpponent)),
        _ => {}
    }
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    (!plural && end(tail).is_empty()).then_some(Sel::All(f))
}

/// "The new targets can't be you or a permanent you control." after a random reselection.
fn new_targets_cant_be(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower)
        .strip_prefix("the new targets can't be ")
        .or_else(|| end(&lower).strip_prefix("the new target can't be "))
    else {
        return false;
    };
    let Some(parts) = r.split(" or ").map(excluded).collect::<Option<Vec<Sel>>>() else {
        return false;
    };
    let slot = match prev {
        Effect::ReselectTargetsAtRandom { except, .. } => except,
        Effect::Seq(v) => match v.last_mut() {
            Some(Effect::ReselectTargetsAtRandom { except, .. }) => except,
            _ => return false,
        },
        _ => return false,
    };
    if slot.is_some() {
        return false;
    }
    *slot = Some(if parts.len() == 1 {
        parts.into_iter().next().unwrap_or(Sel::None)
    } else {
        Sel::Union(parts)
    });
    true
}

inventory::submit! { FollowupPattern { name: "spell targets 6: the new targets can't be", priority: 45, apply: new_targets_cant_be } }
