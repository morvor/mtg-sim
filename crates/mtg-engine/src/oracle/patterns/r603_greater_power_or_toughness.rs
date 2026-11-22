//! An evolve-like intervening "if" clause (CR 603.4) comparing the creature that triggered
//! the ability with its source: "Whenever another creature you control enters, if that
//! creature has greater power or toughness than ~, put an oil counter on ~." (Evolving
//! Adaptive), "... if it has greater power or toughness than ~, ..." (Hulkling,
//! Burgeoning Bruiser). As for evolve (CR 702.100a), power is compared to power and toughness to
//! toughness, as the ability triggers and again as it resolves, using the entered
//! creature's last known information if it has left the battlefield.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn greater_power_or_toughness(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("that creature has greater ")
        .or_else(|| c.strip_prefix("it has greater "))?;
    let r = r.strip_suffix(" than ~")?;
    let entered = || Box::new(Sel::TriggerObject);
    let this = || Box::new(Sel::This);
    let power = || Condition::Compare(Value::PowerOf(entered()), Cmp::Gt, Value::PowerOf(this()));
    let toughness = || {
        Condition::Compare(
            Value::ToughnessOf(entered()),
            Cmp::Gt,
            Value::ToughnessOf(this()),
        )
    };
    match r {
        "power" => Some(power()),
        "toughness" => Some(toughness()),
        "power or toughness" => Some(Condition::Or(vec![power(), toughness()])),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "r603 that creature/it has greater power or toughness than ~", priority: 100, parse: greater_power_or_toughness } }
