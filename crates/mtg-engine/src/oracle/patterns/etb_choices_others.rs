//! Replacement effects that modify how *other* permanents enter (CR 614.1d, 614.12):
//! "Creatures your opponents control enter tapped."

use super::{AbilityPattern, ConditionPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// An entering object isn't on the battlefield yet, so "permanent" in the subject means
/// any object that would become a permanent.
fn entering_filter(f: Filter) -> Filter {
    match f {
        Filter::Permanent => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(entering_filter).collect()),
        Filter::Or(v) => Filter::Or(v.into_iter().map(entering_filter).collect()),
        other => other,
    }
}

/// A plural subject that may list several kinds of objects sharing a controller suffix:
/// "creatures and nonbasic lands your opponents control" means creatures your opponents
/// control and nonbasic lands your opponents control.
fn subject_list(subj: &str) -> Option<Filter> {
    let (heads, suffix) = [
        " you control",
        " your opponents control",
        " an opponent controls",
    ]
    .iter()
    .find_map(|sfx| subj.strip_suffix(sfx).map(|h| (h, *sfx)))
    .unwrap_or((subj, ""));
    let mut parts = Vec::new();
    for p in heads
        .split(", and ")
        .flat_map(|p| p.split(", or "))
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" and "))
        .flat_map(|p| p.split(" or "))
    {
        let (f, plural, tail) = parse_object_phrase(p.trim())?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        parts.push(f);
    }
    let head = match parts.len() {
        0 => return None,
        1 => parts.pop().unwrap(),
        _ => Filter::Or(parts),
    };
    if suffix.is_empty() {
        return Some(head);
    }
    // Parse the controller suffix on a neutral head noun ("card" = any object).
    let probe = format!("card{suffix}");
    let (sf, _, tail) = parse_object_phrase(&probe)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Filter::and(vec![head, sf]))
}

/// "[objects] enter tapped", e.g. "Artifacts, creatures, and lands your opponents
/// control enter tapped." (a list in the subject is a union).
fn others_enter_tapped(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let subj = l.strip_suffix(" enter tapped")?;
    if subj.starts_with('~') || subj.contains(" this turn") {
        return None;
    }
    let f = subject_list(subj)?;
    let def = ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(entering_filter(f)),
        action: ReplacementAction::EnterTapped,
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        block,
    )])
}

// ---------------------------------------------------------------------------
// Day and night (CR 731)
// ---------------------------------------------------------------------------

/// "If it's neither day nor night, it becomes day as ~ enters." — a replacement effect
/// applied as the permanent enters (CR 614.1c, 731.1).
fn day_as_enters(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("if ")?;
    let (c, e) = r.split_once(", ")?;
    let e = e.strip_suffix(" as ~ enters")?;
    let cond = crate::oracle::statics::parse_condition(c, ctx)?;
    let eff = day_night_effect_inner(e)?;
    let def = ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::If {
            cond,
            then: Box::new(eff),
            otherwise: Box::new(Effect::Noop),
        })),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        block,
    )])
}

fn day_night_effect_inner(l: &str) -> Option<Effect> {
    match end(l) {
        "it becomes day" => Some(Effect::SetDayNight { day: true }),
        "it becomes night" => Some(Effect::SetDayNight { day: false }),
        _ => None,
    }
}

fn day_night_effect(l: &str, _b: &mut Builder) -> Option<Effect> {
    day_night_effect_inner(l)
}

fn day_night_condition(c: &str) -> Option<Condition> {
    match end(c) {
        "it's neither day nor night" => Some(Condition::And(vec![
            Condition::Not(Box::new(Condition::IsDay)),
            Condition::Not(Box::new(Condition::IsNight)),
        ])),
        _ => None,
    }
}

/// "Whenever day becomes night or night becomes day" (CR 731.1a).
fn day_night_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "day becomes night or night becomes day").then_some((
        TriggerCond::DayNightChanges,
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! {
    AbilityPattern { name: "others enter tapped", priority: 50, parse: others_enter_tapped }
}
inventory::submit! {
    AbilityPattern { name: "it becomes day as ~ enters", priority: 50, parse: day_as_enters }
}
inventory::submit! {
    EffectPattern { name: "it becomes day/night", priority: 100, parse: day_night_effect }
}
inventory::submit! {
    ConditionPattern { name: "neither day nor night", priority: 100, parse: day_night_condition }
}
inventory::submit! {
    TriggerPattern { name: "day becomes night or night becomes day", priority: 100, parse: day_night_trigger }
}
