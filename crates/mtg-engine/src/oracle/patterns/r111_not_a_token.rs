//! "if it's not a token" / "if it isn't a token" as the intervening "if" of a triggered
//! ability about the object itself ("When this creature dies, if it's not a token, create a
//! token that's a copy of it", CR 603.4, 111.1): checked against the object's last known
//! information when it has left the battlefield (CR 603.10a).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn not_a_token(c: &str) -> Option<Condition> {
    match end(c) {
        "it's not a token" | "it isn't a token" | "~ isn't a token" | "~ is not a token" => Some(
            Condition::Not(Box::new(Condition::SelMatches(Sel::This, Filter::Token))),
        ),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "it's not a token", priority: 100, parse: not_a_token } }
