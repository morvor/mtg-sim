//! Turns a player has taken ("your first, second, or third turn of the game"): each
//! player's count of the turns they've taken this game, including the current one
//! ([`crate::game::Player::turns_taken`], [`Value::TurnsTaken`]), counted as each turn
//! begins (CR 500.1). It counts the player's own turns, extra turns included, not the
//! turns the game has had (Serra Avenger ruling), and starts afresh when the game is
//! restarted (CR 727).
//!
//! "It's your first, second, or third turn of the game" is "it's your turn and you've
//! taken at most three turns" (the current one included): [`early_turns`].

use crate::ability::*;

const ORDINALS: [&str; 7] = [
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
];

/// "It's your first [, second, ... or Nth] turn of the game".
pub fn early_turns(n: u32) -> Condition {
    Condition::And(vec![
        Condition::YourTurn,
        Condition::Compare(
            Value::TurnsTaken(PlayerRef::You),
            Cmp::Le,
            Value::Const(n as i32),
        ),
    ])
}

/// The N of a condition made by [`early_turns`].
pub fn early_turns_n(c: &Condition) -> Option<u32> {
    match c {
        Condition::And(v) if v.len() == 2 => match (&v[0], &v[1]) {
            (
                Condition::YourTurn,
                Condition::Compare(Value::TurnsTaken(PlayerRef::You), Cmp::Le, Value::Const(n)),
            ) if *n >= 1 && (*n as usize) <= ORDINALS.len() => Some(*n as u32),
            _ => None,
        },
        _ => None,
    }
}

/// "first", "first or second", "first, second, or third", ...
pub fn ordinal_list(n: u32) -> String {
    let w = &ORDINALS[..n as usize];
    match w.len() {
        1 => w[0].to_string(),
        2 => format!("{} or {}", w[0], w[1]),
        k => format!("{}, or {}", w[..k - 1].join(", "), w[k - 1]),
    }
}

/// Parses an ordinal list made by [`ordinal_list`] ("first, second, or third"), followed
/// by the rest of the text.
pub fn parse_ordinal_list(s: &str) -> Option<(u32, &str)> {
    (1..=ORDINALS.len() as u32)
        .rev()
        .find_map(|n| s.strip_prefix(ordinal_list(n).as_str()).map(|r| (n, r)))
}
