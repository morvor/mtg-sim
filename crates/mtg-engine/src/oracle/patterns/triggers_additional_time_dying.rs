//! "If a creature dying causes a triggered ability of a permanent you control to trigger,
//! that ability triggers an additional time." (Teysa Karlov; Drivnod, Carnage Dominus):
//! CR 603.2d, for abilities whose trigger event is such an object dying (a cause of
//! [`TriggerCond::Dies`]), including leaves-the-battlefield abilities of permanents that
//! die at the same time (CR 603.10a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn dying_triggers_additional_time(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let objects = end(l)
        .strip_prefix("if ")?
        .strip_suffix(
            " dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time",
        )?;
    let objects = objects
        .strip_prefix("a ")
        .or_else(|| objects.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(objects)?;
    if plural || !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
            sources: Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            cause: Some(Box::new(TriggerCond::Dies(f))),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r603.2d [objects] dying causes a triggered ability to trigger an additional time", priority: 100, parse: dying_triggers_additional_time } }
