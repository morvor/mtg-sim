//! "If excess damage was dealt to a permanent this way, create that many tapped Treasure
//! tokens." (Bottle-Cap Blast): the excess damage the previous damage instruction dealt
//! (CR 120.10: beyond lethal damage for a creature, beyond its loyalty for a planeswalker,
//! beyond its defense for a battle), recorded as the damage is dealt
//! (`vars::EXCESS`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn if_excess_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if excess damage was dealt ")?;
    let r = r
        .strip_prefix("to a permanent ")
        .or_else(|| r.strip_prefix("to that permanent "))
        .unwrap_or(r);
    let r = r.strip_prefix("this way, ")?;
    // "discover X, where X is that excess damage": the same amount.
    let defined = r.contains(", where x is that excess damage");
    let r = &r.replace(", where x is that excess damage", "");
    // "that many" is the amount of excess damage.
    let x_given = r.split(' ').any(|w| w == "x");
    if x_given && (!defined || r.contains("that many")) {
        return None;
    }
    let e = parse_clause(&r.replace("that many", "x"), b)?;
    let excess = Value::Var(vars::EXCESS);
    let e = super::a701_action_triggers::substitute_x(&e, &excess)?;
    Some(Effect::If {
        cond: Condition::Compare(excess, Cmp::Gt, Value::c(0)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if excess damage was dealt this way, [effect with that many]", priority: 100, parse: if_excess_this_way } }
