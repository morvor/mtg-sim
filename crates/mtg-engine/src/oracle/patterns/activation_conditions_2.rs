//! Conditions in activation restrictions (CR 602.5b):
//!
//! * "you've controlled ~ continuously since the beginning of your most recent turn"
//!   (Rocket Launcher; see `kw/control_since_your_turn.rs`);
//! * "you're attacking two or more opponents" (Nemesis Phoenix): the opponents you're
//!   attacking this combat;
//! * "there were three or more card types among the sacrificed permanents" (Baba Lysaga,
//!   Night Witch): the permanents the cost sacrificed, as they last existed.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn controlled_since_your_turn(c: &str) -> Option<Condition> {
    (end(c) == "you've controlled ~ continuously since the beginning of your most recent turn")
        .then(|| {
            Condition::Custom(
                crate::kw::control_since_your_turn::CONTROLLED_SINCE_YOUR_TURN.into(),
            )
        })
}

inventory::submit! { ConditionPattern { name: "activation conditions: you've controlled ~ continuously since your most recent turn began", priority: 100, parse: controlled_since_your_turn } }

/// "you're attacking two or more opponents".
fn attacking_opponents(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you're attacking ")?;
    let (cmp, n, rest) = super::statics_conditions::amount_cmp(r)?;
    if !matches!(rest.trim(), "opponents" | "opponent") {
        return None;
    }
    // You're the attacking player (the active player, CR 506.2).
    Some(Condition::And(vec![
        Condition::YourTurn,
        Condition::Compare(
            Value::Custom(crate::kw::value_counts::OPPONENTS_BEING_ATTACKED.into()),
            cmp,
            n,
        ),
    ]))
}

inventory::submit! { ConditionPattern { name: "activation conditions: you're attacking N or more opponents", priority: 100, parse: attacking_opponents } }

/// "there were three or more card types among the sacrificed permanents".
fn types_among_sacrificed(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("there were ")?;
    let (cmp, n, rest) = super::statics_conditions::amount_cmp(r)?;
    if rest.trim() != "card types among the sacrificed permanents" {
        return None;
    }
    Some(Condition::Compare(
        Value::DistinctAmong(Among::CardTypes, Box::new(Sel::Var(vars::SACRIFICED))),
        cmp,
        n,
    ))
}

inventory::submit! { ConditionPattern { name: "activation conditions: there were N or more card types among the sacrificed permanents", priority: 100, parse: types_among_sacrificed } }
