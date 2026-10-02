//! "an opponent cast a [kind of] spell this turn" / "an opponent has cast a [kind of]
//! spell this turn": "Take an extra turn after this one if an opponent cast a blue spell
//! this turn." (Seedtime). Casting a spell only requires it to be put on the stack and
//! its costs paid (CR 601.2), so a spell that hasn't resolved or was countered counts.
//! Also "you've cast a [kind of] spell this turn" ("This creature can't be blocked if
//! you've cast a historic spell this turn.", Relic Runner).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};

fn opponent_cast_this_turn(c: &str) -> Option<Condition> {
    let c = end(c);
    let (who, r) = if let Some(r) = c
        .strip_prefix("an opponent cast ")
        .or_else(|| c.strip_prefix("an opponent has cast "))
    {
        (PlayerRef::EachOpponent, r)
    } else {
        (PlayerRef::You, c.strip_prefix("you've cast ")?)
    };
    let desc = r.strip_suffix(" this turn")?;
    let desc = desc
        .strip_prefix("a ")
        .or_else(|| desc.strip_prefix("an "))?;
    let (filter, plural, rest) = parse_object_phrase(desc)?;
    if plural || !rest.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::SpellsCastThisTurn(who, filter),
        Cmp::Gt,
        Value::c(0),
    ))
}

inventory::submit! { ConditionPattern { name: "an opponent cast a [kind of] spell this turn", priority: 100, parse: opponent_cast_this_turn } }
