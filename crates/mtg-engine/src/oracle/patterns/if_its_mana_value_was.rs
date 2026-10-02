//! "Return target nonland permanent you don't control to its owner's hand. If its mana
//! value was 2 or less, scry 2." (Perilous Voyage), "Counter target spell. If that
//! spell's mana value was 3 or less, proliferate." (Reject Imperfection): the mana value
//! of the object the earlier sentence acted on, as it last existed before that action
//! (CR 608.2h: its last known information once it has changed zones).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

fn if_its_mana_value_was(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let r = ["its ", "that permanent's ", "that spell's ", "that creature's "]
        .iter()
        .find_map(|p| r.strip_prefix(p))?;
    let r = r.strip_prefix("mana value was ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    let r = r.trim_start();
    let (cmp, r) = if let Some(r) = r.strip_prefix("or less") {
        (Cmp::Le, r)
    } else if let Some(r) = r.strip_prefix("or greater") {
        (Cmp::Ge, r)
    } else {
        return None;
    };
    let rest = r.strip_prefix(", ")?;
    // "Its" must be an object an earlier sentence targeted.
    let Sel::Target(slot) = b.it else {
        return None;
    };
    let then = parse_clause(rest, b)?;
    Some(Effect::If {
        cond: Condition::SelMatches(
            Sel::Target(slot),
            Filter::ManaValue(cmp, Box::new(Value::c(n))),
        ),
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if its mana value was N or less, [effect]", priority: 100, parse: if_its_mana_value_was } }
