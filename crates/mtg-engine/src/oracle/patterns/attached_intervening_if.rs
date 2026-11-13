//! Triggered abilities of Auras and Equipment with an intervening "if" clause about the
//! enchanted or equipped object (CR 603.4) whose effect doesn't refer back to that object:
//! "At the beginning of the end step, if enchanted creature's power is 4 or greater,
//! destroy ~." (Arachnus Web), "At the beginning of your end step, if enchanted
//! creature's power is 4 or greater, sacrifice ~." (Domestication). The condition is
//! checked as the ability would trigger and again as it resolves.
//!
//! The generic intervening-if conditions leave "enchanted ..." conditions to static
//! abilities, because an effect after such a clause usually says "it", which the effect
//! parser can't tie to the enchanted object; this pattern accepts only effects without
//! such a reference.

use super::statics_conditions::parse_static_condition;
use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn attached_intervening_if(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if !(t.starts_with("When ") || t.starts_with("Whenever ") || t.starts_with("At ")) {
        return None;
    }
    let (trigger, rest) = t.split_once(", if ")?;
    let lower = rest.to_lowercase();
    if !(lower.starts_with("enchanted ") || lower.starts_with("equipped ")) {
        return None;
    }
    let (cond_text, effect) = lower.split_once(", ")?;
    // The effect must not refer to the object the condition is about.
    let words: Vec<&str> = effect
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .collect();
    if words.iter().any(|w| {
        matches!(
            *w,
            "it" | "its" | "it's" | "that" | "enchanted" | "equipped" | "they" | "their"
        )
    }) {
        return None;
    }
    let cond = match greatest_power(cond_text) {
        Some(c) => c,
        None => parse_static_condition(cond_text, None, ctx)?.0,
    };
    let effect_orig = &rest[rest.len() - effect.len()..];
    let without = format!("{trigger}, {effect_orig}");
    let mut ability = crate::oracle::triggers::parse_triggered(&without, ctx)?;
    let a = std::sync::Arc::make_mut(&mut ability);
    let AbilityKind::Triggered(tr) = &mut a.kind else {
        return None;
    };
    tr.intervening_if = Some(match tr.intervening_if.take() {
        Some(c) => Condition::And(vec![cond, c]),
        None => cond,
    });
    a.text = t.into();
    Some(vec![ability])
}

/// "enchanted permanent is a creature with the greatest power among creatures on the
/// battlefield" (Historian's Wisdom): it's a creature, and no creature among them has
/// greater power (ties count).
fn greatest_power(c: &str) -> Option<Condition> {
    let r = c
        .strip_prefix("enchanted permanent is a creature with the greatest power among ")
        .or_else(|| c.strip_prefix("enchanted creature has the greatest power among "))?;
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(r)?;
    if !plural || !crate::oracle::phrases::end(tail).is_empty() {
        return None;
    }
    Some(Condition::And(vec![
        Condition::SelMatches(Sel::AttachedTo, Filter::creature()),
        Condition::Compare(
            Value::PowerOf(Box::new(Sel::AttachedTo)),
            Cmp::Ge,
            Value::GreatestPower(f),
        ),
    ]))
}

inventory::submit! { AbilityPattern { name: "trigger, if enchanted/equipped [object] [state], [effect not about it]", priority: 70, parse: attached_intervening_if } }
