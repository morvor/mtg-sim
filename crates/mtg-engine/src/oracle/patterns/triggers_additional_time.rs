//! "If a triggered ability of a Dwarf you control triggers, that ability triggers an
//! additional time." (Bifur, Melodic Rider, with "As long as you have an enduring story,";
//! Katara, the Fearless; Splinter, Radical Rat; ...): CR 603.2d, for any trigger event
//! ([`StaticEffect::AdditionalTrigger`] with no cause); "If an artifact or creature
//! entering causes a triggered ability of a permanent you control to trigger, that ability
//! triggers an additional time." (Panharmonicon, Yarok, the Desecrated): only abilities
//! whose trigger event is such a permanent entering (a cause of `EntersBattlefield`).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn triggers_additional_time(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if a triggered ability of ")?;
    let objects = r
        .strip_suffix(" triggers, that ability triggers an additional time")
        .or_else(|| r.strip_suffix(" triggers, it triggers an additional time"))?;
    // "another Elemental you control": other than the source.
    let (other, objects) = match objects.strip_prefix("another ") {
        Some(o) => (true, o),
        None => (false, objects),
    };
    // "equipped creature" / "enchanted creature" without an article is the creature this
    // permanent is attached to (Wizard's Staff), not any equipped or enchanted creature.
    if !other && matches!(objects, "equipped creature" | "enchanted creature") {
        return Some(vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
                sources: Filter::and(vec![Filter::AttachedToSource, Filter::creature()]),
                cause: None,
            })),
            text,
        )]);
    }
    if ["equipped ", "enchanted ", "fortified "]
        .iter()
        .any(|p| objects.starts_with(p))
    {
        return None;
    }
    let objects = objects
        .strip_prefix("a ")
        .or_else(|| objects.strip_prefix("an "))
        .unwrap_or(objects);
    let (f, plural, tail) = parse_object_phrase(objects)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    // Objects on the battlefield unless the phrase names another zone ("spell").
    let f = if f.zone().is_none() {
        Filter::and(vec![f, Filter::Permanent])
    } else {
        f
    };
    let sources = if other {
        Filter::and(vec![Filter::Other, f])
    } else {
        f
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
            sources,
            cause: None,
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r603.2d a triggered ability of [objects] triggers an additional time", priority: 100, parse: triggers_additional_time } }

/// "If [objects] entering [the battlefield] causes a triggered ability of a permanent you
/// control to trigger, that ability triggers an additional time." (CR 603.2d): the event
/// causing the ability to trigger must be such an object entering (CR 603.6a).
fn entering_triggers_additional_time(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l)
        .strip_prefix("if ")?
        .strip_suffix(
            " causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time",
        )?;
    let objects = r
        .strip_suffix(" entering the battlefield")
        .or_else(|| r.strip_suffix(" entering"))?;
    let objects = objects
        .strip_prefix("a ")
        .or_else(|| objects.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(objects)?;
    // Objects entering the battlefield: permanents.
    if plural || !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
            sources: Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            cause: Some(Box::new(TriggerCond::EntersBattlefield(f))),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r603.2d [objects] entering causes a triggered ability to trigger an additional time", priority: 100, parse: entering_triggers_additional_time } }
