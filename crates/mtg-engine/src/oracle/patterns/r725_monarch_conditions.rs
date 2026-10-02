//! Conditions about who the monarch is (CR 725): "if an opponent is the monarch" (Queen
//! Marchesa). ("If you're the monarch" is a core condition.)

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::*;

fn parse(c: &str) -> Option<Condition> {
    match end(c) {
        "an opponent is the monarch" => Some(Condition::PlayerMatches(
            PlayerRef::EachOpponent,
            PlayerFilter::Monarch,
        )),
        _ => None,
    }
}

inventory::submit! {
    ConditionPattern { name: "r725 an opponent is the monarch", priority: 100, parse }
}
