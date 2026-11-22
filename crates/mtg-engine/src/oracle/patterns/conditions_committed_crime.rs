//! "you've committed a crime this turn" (CR 700.13; Take for a Ride, Oko, the Ringleader):
//! see `kw/committed_crime.rs`.

use super::ConditionPattern;
use crate::ability::Condition;

fn committed_crime(l: &str) -> Option<Condition> {
    matches!(
        l,
        "you've committed a crime this turn" | "you have committed a crime this turn"
    )
    .then(|| Condition::Custom(crate::kw::committed_crime::COMMITTED_CRIME_THIS_TURN.into()))
}

inventory::submit! { ConditionPattern { name: "you've committed a crime this turn", priority: 100, parse: committed_crime } }
