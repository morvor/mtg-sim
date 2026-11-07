//! Conditions on controlling the creature with the greatest power: "draw a card if you
//! control a creature with the greatest power among creatures on the battlefield" (Primal
//! Empathy, High Score), "draw two cards if you control the creature with the greatest
//! power or tied for the greatest power" (Thickest in the Thicket). A creature tied for the
//! greatest power counts, whoever controls the others. Checked as the ability resolves
//! (CR 608.2c).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn control_greatest_power(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    match r {
        "a creature with the greatest power among creatures on the battlefield"
        | "the creature with the greatest power or tied for the greatest power"
        | "a creature with the greatest power or tied for the greatest power" => {}
        _ => return None,
    }
    let greatest = Value::GreatestPower(Filter::creature());
    Some(Condition::Exists(
        Filter::and(vec![
            Filter::creature(),
            Filter::Power(Cmp::Ge, Box::new(greatest)),
        ])
        .you_control(),
    ))
}

inventory::submit! { ConditionPattern { name: "you control the creature with the greatest power", priority: 100, parse: control_greatest_power } }
