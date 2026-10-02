//! Conditions about permanents that died this turn (CR 700.4: "dies" means is put into a
//! graveyard from the battlefield), checked against the permanents as they last existed
//! on the battlefield (their last known information):
//! * "no creatures died this turn" (Titan Hunter);
//! * "another Human died under your control this turn" (White Glove Gourmand), "a modified
//!   creature died under your control this turn" (Intermediate Chirography);
//! * "an artifact or creature was put into a graveyard from the battlefield this turn"
//!   (Ichor Shade).
//!
//! ("A creature died this turn" and "a creature died under your control this turn" are
//! parsed by `trigger_conditions`.)

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::*;

fn parse(c: &str) -> Option<Condition> {
    let c = end(c);
    if c == "no creatures died this turn" {
        return Some(Condition::Not(Box::new(Condition::Compare(
            Value::CreaturesDiedThisTurn,
            Cmp::Gt,
            Value::c(0),
        ))));
    }
    let r = c.strip_prefix("a ").or_else(|| c.strip_prefix("an "));
    let (r, other) = match r {
        Some(r) => (r, false),
        None => (c.strip_prefix("another ")?, true),
    };
    let (phrase, yours) = if let Some(p) = r.strip_suffix(" died under your control this turn") {
        (p, true)
    } else if let Some(p) = r.strip_suffix(" died this turn") {
        (p, false)
    } else {
        (
            r.strip_suffix(" was put into a graveyard from the battlefield this turn")?,
            false,
        )
    };
    let (f, plural, tail) = parse_object_phrase(phrase)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let mut parts = vec![f];
    if other {
        parts.push(Filter::Other);
    }
    if yours {
        parts.push(Filter::ControlledBy(PlayerRel::You));
    }
    // Whether a dies event of this turn matches, regardless of whether anything triggered
    // on it (CR 603.1b).
    Some(Condition::AllTriggerConditionsThisTurn(vec![TriggerCond::Dies(
        Filter::and(parts),
    )]))
}

inventory::submit! {
    ConditionPattern { name: "r700 died this turn", priority: 110, parse }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        for c in [
            "no creatures died this turn",
            "another Human died under your control this turn",
            "a modified creature died under your control this turn",
            "an artifact or creature was put into a graveyard from the battlefield this turn",
        ] {
            assert!(parse(c).is_some(), "{c}");
        }
    }
}
