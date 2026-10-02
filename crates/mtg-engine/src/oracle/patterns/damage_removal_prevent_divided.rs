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
    if !matches!(n, Value::Const(_)) {
        return None;
    }
    let r = r
        .trim_start()
        .strip_prefix("damage that would be dealt this turn to ")?;
    let (who, rest) = r.split_once(", divided as you choose")?;
    if !end(rest).is_empty() {
        return None;
    }
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
    spec.min = if any_number { 0 } else { 1 };
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
