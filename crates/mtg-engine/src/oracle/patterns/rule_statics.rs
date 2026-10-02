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

/// "it's your first, second, or third turn of the game".
fn early_turn_condition(c: &str) -> Option<Condition> {
    use crate::rule_statics::turns_taken::*;
    let (n, rest) = parse_ordinal_list(c.trim().strip_prefix("it's your ")?)?;
    (rest == " turn of the game").then(|| early_turns(n))
}

/// "You can't cast ~ during your first, second, or third turns of the game." (Serra
/// Avenger): a restriction on casting the card itself, wherever it's cast from (CR 601.3);
/// casting it during another player's turn isn't restricted.
fn cant_cast_during_early_turns(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::rule_statics::turns_taken::*;
    let r = end(l)
        .trim()
        .strip_prefix("you can't cast ~ during your ")?;
    let (n, rest) = parse_ordinal_list(r)?;
    if rest != " turns of the game" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::CastOnlyIf(Condition::Not(Box::new(
        early_turns(n),
    ))));
    s.zone = FunctionZone::Anywhere;
    Some(static_ability(s, text))
}

inventory::submit! { ConditionPattern { name: "it's your first, second, or third turn of the game", priority: 60, parse: early_turn_condition } }
inventory::submit! { StaticPattern { name: "you can't cast ~ during your first turns", priority: 100, parse: cant_cast_during_early_turns } }

/// "Permanents your opponents control can't be turned face up during your turn", "As long
/// as enchanted creature is face down, it can't be turned face up" (CR 708.7).
fn cant_be_turned_face_up(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l).trim();
    // "As long as [permanent] is face down, it can't be turned face up": only a face-down
    // permanent can be turned face up anyway.
    if let Some(r) = l.strip_prefix("as long as ") {
        let subject = r.strip_suffix(" is face down, it can't be turned face up")?;
        let f = match subject {
            "enchanted creature" | "enchanted permanent" => Filter::AttachedToSource,
            _ => permanents_phrase(subject)?,
        };
        return Some(static_ability(
            StaticAbility::new(StaticEffect::Restriction(Restriction::CantTurnFaceUp(
                Filter::and(vec![f, Filter::FaceDown]),
            ))),
            text,
        ));
    }
    let (subject, during_your_turn) = match l.strip_suffix(" can't be turned face up during your turn") {
        Some(s) => (s, true),
        None => (l.strip_suffix(" can't be turned face up")?, false),
    };
    let f = permanents_phrase(subject)?;
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantTurnFaceUp(f)));
    if during_your_turn {
        s.condition = Some(Condition::YourTurn);
    }
    Some(static_ability(s, text))
}

inventory::submit! { StaticPattern { name: "can't be turned face up", priority: 100, parse: cant_be_turned_face_up } }
