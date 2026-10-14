//! Pack tactics (an ability word): "Whenever ~ attacks, if you attacked with creatures
//! with total power 6 or greater this combat, ..." — the intervening "if" (CR 603.4)
//! compares the total power of the creatures you're attacking with to N, both as the
//! ability triggers and as it resolves.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number};

fn attacked_with_total_power(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you attacked with creatures with total power ")?;
    let (n, rest) = parse_number(r)?;
    n.as_const()?;
    if end(rest) != "or greater this combat" {
        return None;
    }
    let attackers = Filter::and(vec![Filter::creature().you_control(), Filter::Attacking]);
    Some(Condition::Compare(
        Value::PowerOf(Box::new(Sel::All(attackers))),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { ConditionPattern { name: "you attacked with creatures with total power N or greater this combat", priority: 100, parse: attacked_with_total_power } }
