//! Rule-modifying static abilities (see `rule_statics/`):
//!
//! - "Damage isn't removed from [~ / creatures / creatures your opponents control] during
//!   cleanup steps." (an exception to CR 514.2).
//! - "Counters remain on ~ as it moves to any zone other than a player's hand or library."
//!   (an exception to CR 122.2), functioning in every zone.
//! - The condition "[N] or more creatures are damaged" (Case of the Market Melee's "To
//!   solve"): creatures with damage marked on them (CR 120.3e).

use super::{ConditionPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use smol_str::SmolStr;
use crate::oracle::CompileContext;

fn static_ability(s: StaticAbility, text: &str) -> Vec<Ability> {
    vec![AbilityDef::new(AbilityKind::Static(s), text)]
}

/// A whole object phrase naming permanents: "~", "creatures", "creatures your opponents
/// control".
pub(crate) fn permanents_phrase(s: &str) -> Option<Filter> {
    let s = s.trim();
    if s == "~" {
        return Some(Filter::Source);
    }
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(Filter::and(vec![Filter::Permanent, f]))
}

fn damage_not_removed(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let what = end(l)
        .trim()
        .strip_prefix("damage isn't removed from ")?
        .strip_suffix(" during cleanup steps")?;
    let f = permanents_phrase(what)?;
    Some(static_ability(
        StaticAbility::new(StaticEffect::DamageNotRemoved(f)),
        text,
    ))
}

fn counters_remain(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell()
        || end(l).trim()
            != "counters remain on ~ as it moves to any zone other than a player's hand or library"
    {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::CountersRemain);
    s.zone = FunctionZone::Anywhere;
    Some(static_ability(s, text))
}

inventory::submit! { StaticPattern { name: "damage isn't removed during cleanup steps", priority: 100, parse: damage_not_removed } }
inventory::submit! { StaticPattern { name: "counters remain on ~ as it moves", priority: 100, parse: counters_remain } }

/// "three or more creatures are damaged": the number of creatures with damage marked on
/// them is at least N.
fn damaged_count(c: &str) -> Option<Condition> {
    let (n, rest) = parse_number(c)?;
    let noun = rest.trim().strip_prefix("or more ")?.strip_suffix(" are damaged")?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if !plural || !end(tail).trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![
            Filter::Permanent,
            f,
            Filter::Custom(SmolStr::new(crate::rule_statics::cleanup_damage::DAMAGED)),
        ])),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { ConditionPattern { name: "n or more creatures are damaged", priority: 60, parse: damaged_count } }
