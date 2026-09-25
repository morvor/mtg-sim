//! Conditions on player resources and conditional activation:
//!
//! * "an opponent has three or more poison counters" (corrupted), "you have five or more
//!   experience counters", "you have no {E}" (CR 122.1, 107.14);
//! * "Activate only if [condition]." on any activated ability (CR 602.5b): the condition
//!   is checked as the ability is activated.

use super::{AbilityPattern, ConditionPattern};
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "N or more", "N or fewer", "no", "one or more", "exactly N", "N".
fn amount_cmp(s: &str) -> Option<(Cmp, Value, &str)> {
    if let Some(r) = strip(s, "no ") {
        return Some((Cmp::Eq, Value::c(0), r));
    }
    let (exact, s) = match strip(s, "exactly ") {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (n, r) = parse_number(s)?;
    if matches!(n, Value::X) {
        return None;
    }
    if exact {
        return Some((Cmp::Eq, n, r));
    }
    if let Some(r) = strip(r, "or more ") {
        return Some((Cmp::Ge, n, r));
    }
    if let Some(r) = strip(r, "or fewer ").or_else(|| strip(r, "or less ")) {
        return Some((Cmp::Le, n, r));
    }
    None
}

/// "poison counters", "experience counter", "{E}" → the counter kind.
fn player_counter_kind(s: &str) -> Option<CounterKind> {
    let s = end(s);
    if s == "{e}" {
        return Some(crate::types::counters::ENERGY.into());
    }
    let (kind, rest) = s.split_once(' ')?;
    if !matches!(rest, "counter" | "counters") || kind.contains('/') {
        return None;
    }
    if !matches!(kind, "poison" | "experience" | "energy" | "rad" | "ticket") {
        return None;
    }
    Some(kind.into())
}

use crate::types::CounterKind;

/// "an opponent has three or more poison counters" (the most any opponent has), "you
/// have five or more experience counters", "you have no {E}".
fn player_counter_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let (value, r): (fn(CounterKind) -> Value, &str) =
        if let Some(r) = c.strip_prefix("an opponent has ") {
            (
                |k| Value::Custom(format!("max_opponent_counters:{k}").into()),
                r,
            )
        } else if let Some(r) = c.strip_prefix("you have ") {
            (|k| Value::PlayerCounters(PlayerRef::You, k), r)
        } else {
            return None;
        };
    let (cmp, n, r) = amount_cmp(r)?;
    let kind = player_counter_kind(r)?;
    // "an opponent has no poison counters" isn't "the most any opponent has is 0".
    if c.starts_with("an opponent") && !matches!(cmp, Cmp::Ge) {
        return None;
    }
    Some(Condition::Compare(value(kind), cmp, n))
}

inventory::submit! { ConditionPattern { name: "counters_resources: player counters", priority: 100, parse: player_counter_condition } }

/// "[cost]: [effect]. Activate only if [condition]." — also "... only if [condition] and
/// only once each turn / and only as a sorcery". The condition must hold as the ability
/// is activated (CR 602.5b).
fn activate_only_if(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let (cost_s, eff_s) = crate::oracle::split_cost(text)?;
    let sentences = crate::oracle::effects::split_sentences(eff_s);
    let i = sentences
        .iter()
        .position(|s| s.to_lowercase().starts_with("activate only if "))?;
    let lower = sentences[i].to_lowercase();
    let cond_s = end(&lower).strip_prefix("activate only if ")?;
    let mut extra = None;
    let mut cond_s = cond_s;
    for (suffix, sentence) in [
        (" and only once each turn", "Activate only once each turn."),
        (" and only as a sorcery", "Activate only as a sorcery."),
        (
            " and only during your turn",
            "Activate only during your turn.",
        ),
    ] {
        if let Some(c) = cond_s.strip_suffix(suffix) {
            cond_s = c;
            extra = Some(sentence);
        }
    }
    // Where the ability functions (CR 113.6): "Activate only if ~ is in your graveyard"
    // means it's activated from the graveyard; other conditions about where the object
    // is aren't handled here.
    let from_graveyard = cond_s == "~ is in your graveyard";
    if !from_graveyard
        && ["~ is ", "~ isn't ", "this card is "]
            .iter()
            .any(|p| cond_s.contains(p))
        && ["graveyard", "hand", "exile", "library", "battlefield"]
            .iter()
            .any(|z| cond_s.contains(z))
    {
        return None;
    }
    let cond = crate::oracle::statics::parse_condition(cond_s, ctx)?;
    let mut rest: Vec<String> = sentences
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != i)
        .map(|(_, s)| s.clone())
        .collect();
    if rest.is_empty() {
        return None;
    }
    if let Some(e) = extra {
        rest.push(e.to_string());
    }
    let new_block = format!("{cost_s}: {}", rest.join(" "));
    let mut abilities = crate::oracle::parse_ability(&new_block, ctx)?;
    if abilities.len() != 1 {
        return None;
    }
    let a = abilities.pop()?;
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    let mut act = act.clone();
    if from_graveyard {
        act.zone = FunctionZone::Graveyard;
    }
    act.condition = Some(match act.condition.take() {
        Some(c) => Condition::And(vec![c, cond]),
        None => cond,
    });
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "counters_resources: activate only if", priority: 100, parse: activate_only_if } }
