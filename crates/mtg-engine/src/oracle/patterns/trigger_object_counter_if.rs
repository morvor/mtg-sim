//! A triggered ability with an intervening "if it has ... counter(s) on it" clause about the
//! object the trigger event refers to: "Whenever a creature you control enters, if it has
//! one or more oil counters on it, put an oil counter on it." (Ichorplate Golem). The
//! condition is checked when the event happens and again on resolution (CR 603.4). (The
//! same clause about the source itself is `counters_resources_counters.rs`'s.)

use super::counters_resources_counters::{amount_cmp, kind_then_on};
use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn trigger_object_counter_if(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    if text.len() != lower.len() || !(lower.starts_with("when ") || lower.starts_with("whenever "))
    {
        return None;
    }
    let i = lower.find(", if it has ")?;
    let cond_s = &lower[..i];
    let rest = &lower[i + ", if it has ".len()..];
    let (_, it, _) = crate::oracle::triggers::parse_trigger_condition(cond_s)?;
    if !matches!(it, Sel::TriggerObject) {
        return None;
    }
    let (clause, _) = rest.split_once(", ")?;
    let (cmp, n, c) = amount_cmp(clause)?;
    let (kind, c) = kind_then_on(c)?;
    if end(c) != "it" {
        return None;
    }
    // The ability without the clause, in the original case.
    let effect_at = i + ", if it has ".len() + clause.len() + ", ".len();
    if !text.is_char_boundary(i) || !text.is_char_boundary(effect_at) {
        return None;
    }
    let without = format!("{}, {}", &text[..i], &text[effect_at..]);
    let a = crate::oracle::triggers::parse_triggered(&without, ctx)?;
    let AbilityKind::Triggered(mut ta) = a.kind.clone() else {
        return None;
    };
    if ta.intervening_if.is_some() {
        return None;
    }
    ta.intervening_if = Some(Condition::Compare(
        Value::CountersOn(Box::new(Sel::TriggerObject), kind),
        cmp,
        n,
    ));
    Some(vec![AbilityDef::new(AbilityKind::Triggered(ta), block)])
}

inventory::submit! { AbilityPattern { name: "trigger: if it (the event's object) has counters on it", priority: 100, parse: trigger_object_counter_if } }
