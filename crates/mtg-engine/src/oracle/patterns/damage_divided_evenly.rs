//! "~ deals X damage divided evenly, rounded down, among any number of targets" (Fireball),
//! "~ deals X damage divided evenly, rounded down, among all creatures target opponent
//! controls" (Dwarven Catapult): each recipient is dealt the amount divided by the number
//! of recipients, rounded down. The division happens as it resolves, among the targets
//! still legal then (CR 608.2b) or the objects that match then.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn divided_evenly(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (pre, rest) = l.split_once(" damage divided evenly, rounded down, among ")?;
    let clause = match rest.strip_prefix("all ") {
        Some(objects) => format!("{pre} damage to each {objects}"),
        None => format!("{pre} damage to each of {rest}"),
    };
    let targets = b.targets.len();
    match parse_clause(&clause, b) {
        Some(Effect::DealDamage { source, amount, to }) => Some(Effect::DealDamage {
            source,
            amount: Value::DivBy(
                Box::new(amount),
                Box::new(Value::CountSel(Box::new(to.clone()))),
            ),
            to,
        }),
        _ => {
            b.targets.truncate(targets);
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "damage divided evenly, rounded down, among", priority: 60, parse: divided_evenly } }
