//! "if ~ didn't attack this turn" (Homicidal Brute, Air Nomad Student): the source wasn't
//! declared as an attacker this turn (CR 508.1). The positive "if ~ attacked this turn"
//! is parsed with the other conditions on the source.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn didnt_attack(c: &str) -> Option<Condition> {
    (end(c) == "~ didn't attack this turn").then(|| {
        Condition::Not(Box::new(Condition::SelMatches(
            Sel::This,
            Filter::AttackedThisTurn,
        )))
    })
}

inventory::submit! { ConditionPattern { name: "r508 ~ didn't attack this turn", priority: 100, parse: didnt_attack } }
