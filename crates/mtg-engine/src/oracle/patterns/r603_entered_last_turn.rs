//! "if you had another creature enter the battlefield under your control last turn"
//! (Ephara, God of the Polis): an intervening "if" clause (CR 603.4) about the previous
//! turn's history, evaluated by `kw/entered_last_turn.rs`.

use super::ConditionPattern;
use crate::ability::Condition;
use crate::oracle::phrases::end;

/// `Condition::Custom` name: another creature entered the battlefield under the ability
/// controller's control during the previous turn.
pub const CREATURE_ENTERED_LAST_TURN: &str = "another creature entered under your control last turn";

fn parse(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "you had another creature enter the battlefield under your control last turn"
            | "you had another creature enter under your control last turn"
    )
    .then(|| Condition::Custom(CREATURE_ENTERED_LAST_TURN.into()))
}

inventory::submit! { ConditionPattern { name: "r603: another creature entered under your control last turn", priority: 100, parse } }
