//! "Prevent the next 5 damage that would be dealt this turn to any number of targets,
//! divided as you choose." (Remedy, Angel of Salvation): the division is chosen as the
//! spell is cast or the ability is put on the stack, each target getting at least 1
//! (CR 601.2d, 603.3d); each target gets a prevention shield of its share (CR 615).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_any_target, parse_number};

fn prevent_divided(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("prevent the next ")?;
    let (n, r) = parse_number(r)?;
    let r = r
        .trim_start()
        .strip_prefix("damage that would be dealt this turn to ")?;
    let (who, rest) = r.split_once(", divided as you choose")?;
    // "..., where X is the number of verse counters on ~" (Serra's Hymn): determined as
    // the division is chosen (CR 601.2d).
    let n = match (n, end(rest)) {
        (n @ Value::Const(_), "") => n,
        (Value::X, w) => {
            let v = w.strip_prefix(", where x is ")?;
            let (v, tail) = super::value_grammar::parse_value(v, b)?;
            if !tail.trim().is_empty() {
                return None;
            }
            v
        }
        _ => return None,
    };
    let (any_number, who) = match who.strip_prefix("any number of ") {
        Some(w) => (true, w.replace("targets", "target")),
        None => (false, who.to_string()),
    };
    let who = if any_number && who == "target" {
        "any target".to_string()
    } else {
        who
    };
    let (mut spec, tail) = parse_any_target(&who)?;
    if !end(tail).is_empty() {
        return None;
    }
    // "Any number of targets" may be zero targets (CR 107.1c).
    spec.min = Value::c(if any_number { 0 } else { 1 });
    if any_number {
        spec.max = n.clone();
    }
    spec.divide = Some(n);
    let slot = b.add_target(spec, "targets (divided)");
    Some(Effect::PreventDividedDamage {
        slot,
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: prevent the next N damage divided among targets", priority: 45, parse: prevent_divided } }

/// "If ~ was kicked, prevent the next 6 damage this way instead." (Pollen Remedy): the
/// total divided depends on the condition, known as the division is chosen (CR 601.2b,
/// 601.2d).
fn f_divided_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some((c, r)) = l.strip_prefix("if ").and_then(|r| r.split_once(", prevent the next "))
    else {
        return false;
    };
    let Some((n, tail)) = parse_number(r) else {
        return false;
    };
    if tail.trim() != "damage this way instead" || !matches!(n, Value::Const(_)) {
        return false;
    }
    let Effect::PreventDividedDamage { slot, .. } = &*prev else {
        return false;
    };
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let Some(spec) = b.targets.get_mut(*slot as usize) else {
        return false;
    };
    let Some(old) = spec.divide.clone() else {
        return false;
    };
    let total = Value::If(Box::new(cond), Box::new(n), Box::new(old.clone()));
    if matches!(spec.min, Value::Const(0)) {
        spec.max = total.clone();
    }
    spec.divide = Some(total);
    true
}

inventory::submit! { super::FollowupPattern { name: "damage_removal: if [condition], prevent the next N damage this way instead", priority: 45, apply: f_divided_instead } }
