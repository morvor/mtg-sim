//! Triggers on another ability triggering (CR 603.3b): "Whenever a creature entering under
//! an opponent's control causes a triggered ability of that creature to trigger, you may
//! copy that ability." (Aboleth Spawn), "Whenever a creature you control attacking causes a
//! triggered ability of that creature to trigger, ..." (Firebender Ascension), "Whenever a
//! permanent entering causes a triggered ability to trigger, ..." (Strict Proctor). The
//! body's "that ability" is the ability that triggered.

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};

fn ability_triggered(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (cause, of_that) = if let Some(c) =
        r.strip_suffix(" causes a triggered ability of that creature to trigger")
    {
        (c, true)
    } else {
        (r.strip_suffix(" causes a triggered ability to trigger")?, false)
    };
    let cause = cause
        .strip_prefix("a ")
        .or_else(|| cause.strip_prefix("an "))?;
    // "[objects] entering [under an opponent's control]", "[objects] attacking",
    // "[objects] dying".
    let (objects, kind, controller) = if let Some((o, rest)) = cause.split_once(" entering") {
        let rest = rest.strip_prefix(" the battlefield").unwrap_or(rest);
        let controller = match rest {
            "" => None,
            " under an opponent's control" => Some(PlayerRel::Opponent),
            " under your control" => Some(PlayerRel::You),
            _ => return None,
        };
        (o, 0, controller)
    } else if let Some(o) = cause.strip_suffix(" attacking") {
        (o, 1, None)
    } else if let Some(o) = cause.strip_suffix(" dying") {
        (o, 2, None)
    } else {
        return None;
    };
    let (f, plural, tail) = parse_object_phrase(objects)?;
    if plural || !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    // "of that creature" needs the cause to be about a creature.
    if of_that && objects.split_whitespace().next() != Some("creature") {
        return None;
    }
    let f = match controller {
        Some(p) => Filter::and(vec![f, Filter::ControlledBy(p)]),
        None => f,
    };
    let cause = match kind {
        0 => TriggerCond::EntersBattlefield(f),
        1 => TriggerCond::Attacks(f),
        _ => TriggerCond::Dies(f),
    };
    let trigger = TriggerCond::AbilityTriggered {
        cause: Box::new(cause),
        source: Filter::Any,
    };
    let trigger = if of_that {
        TriggerCond::Where {
            trigger: Box::new(trigger),
            cond: Condition::SelMatches(
                Sel::TriggerSpell,
                Filter::Custom(crate::stack_ability_filters::OF_THE_TRIGGERING_OBJECT.into()),
            ),
        }
    } else {
        trigger
    };
    Some((trigger, Sel::TriggerSpell, PlayerRef::TriggerPlayer))
}

inventory::submit! { TriggerPattern { name: "[objects] entering/attacking/dying causes a triggered ability [of that creature] to trigger", priority: 100, parse: ability_triggered } }
