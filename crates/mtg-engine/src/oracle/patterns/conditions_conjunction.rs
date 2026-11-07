//! "[condition] and [condition]": two conditions that don't refer back to anything, both
//! of which must hold ("Then if ~ is your Ring-bearer and the Ring has tempted you two or
//! more times this game, draw a card.", Frodo, Adventurous Hobbit). Each part is a whole
//! condition of the condition grammar; conditions with referents joined by "and" are the
//! referent grammar's (`conditions_referents`). Tried last, so a condition whose own
//! wording contains "and" ("you control a Swamp and a Mountain") is read as one.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use std::cell::Cell;

thread_local! {
    /// Set while the parts of a conjunction are parsed: a part is a single condition (no
    /// nested conjunctions, which keeps the search linear in the number of "and"s).
    static IN_PART: Cell<bool> = const { Cell::new(false) };
}

fn conjunction(c: &str) -> Option<Condition> {
    if IN_PART.with(|f| f.get()) {
        return None;
    }
    let c = end(c);
    IN_PART.with(|f| f.set(true));
    let r = c.match_indices(" and ").find_map(|(i, _)| {
        let (a, b) = (&c[..i], &c[i + " and ".len()..]);
        let a = crate::oracle::statics::parse_condition_free(a)?;
        let b = crate::oracle::statics::parse_condition_free(b)?;
        Some(Condition::And(vec![a, b]))
    });
    IN_PART.with(|f| f.set(false));
    r
}

inventory::submit! { ConditionPattern { name: "[condition] and [condition]", priority: 2000, parse: conjunction } }
