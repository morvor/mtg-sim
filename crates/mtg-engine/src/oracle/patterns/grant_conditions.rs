//! Conditions of conditional grants ("~ has flying as long as [condition]", CR 611.3a)
//! about what a player did this turn and about the object's own history:
//!
//! - "you've cast an instant or sorcery spell this turn", "you haven't cast a spell this
//!   turn" (spells cast this turn: put on the stack and paid for, CR 601.2i);
//! - "you've committed a crime this turn" (CR 700.13), "you've surveilled this turn"
//!   (CR 701.25);
//! - "you sacrificed a permanent this turn" (CR 701.21);
//! - "an opponent owns a card in exile";
//! - "it was cast" (a permanent that was a spell that was cast, CR 601.2i);
//! - "two or more creatures are blocking it" (CR 509.1).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase, strip};
use crate::types::CardType;

/// "you sacrificed a [card type] this turn": evaluated by `kw/grant_conditions.rs`.
pub const SACRIFICED_THIS_TURN: &str = "you_sacrificed_this_turn:";
/// "you've surveilled this turn": a named action (`Event::Custom`) the controller
/// performed this turn, evaluated by `kw/grant_conditions.rs`.
pub const ACTION_THIS_TURN: &str = "you_did_this_turn:";
/// "you've committed a crime this turn": evaluated by `kw/grant_conditions.rs`.
pub const COMMITTED_CRIME_THIS_TURN: &str = "you_committed_a_crime_this_turn";

fn not(c: Condition) -> Condition {
    Condition::Not(Box::new(c))
}

/// "a/an [spell phrase]" → its filter.
fn one_spell(desc: &str) -> Option<Filter> {
    let desc = desc
        .strip_prefix("a ")
        .or_else(|| desc.strip_prefix("an "))?;
    let (filter, plural, rest) = parse_object_phrase(desc)?;
    if plural || !rest.trim().is_empty() || !desc.split(' ').any(|w| w == "spell") {
        return None;
    }
    Some(filter)
}

fn cast_this_turn(c: &str) -> Option<Condition> {
    let (r, negated) = if let Some(r) = c.strip_prefix("you've cast ") {
        (r, false)
    } else if let Some(r) = c.strip_prefix("you haven't cast ") {
        (r, true)
    } else {
        return None;
    };
    let desc = r.strip_suffix(" this turn")?;
    // "a spell from your hand": where it was cast from (CR 601.2a).
    let (desc, from) = match desc.strip_suffix(" from your hand") {
        Some(d) => (d, Some(ZoneKind::Hand)),
        None => match desc.strip_suffix(" from exile") {
            Some(d) => (d, Some(ZoneKind::Exile)),
            None => (desc, None),
        },
    };
    let mut f = one_spell(desc)?;
    if let Some(z) = from {
        f = Filter::and(vec![f, Filter::CastFrom(z)]);
    }
    let cast = Condition::Compare(
        Value::SpellsCastThisTurn(PlayerRef::You, f),
        Cmp::Ge,
        Value::c(1),
    );
    Some(if negated { not(cast) } else { cast })
}

fn history(c: &str) -> Option<Condition> {
    match c {
        "you've committed a crime this turn" => {
            return Some(Condition::Custom(COMMITTED_CRIME_THIS_TURN.into()))
        }
        "you haven't committed a crime this turn" => {
            return Some(not(Condition::Custom(COMMITTED_CRIME_THIS_TURN.into())))
        }
        // CR 701.25, 701.22: the keyword actions emit their event after the process.
        "you've surveilled this turn" => {
            return Some(Condition::Custom(
                format!("{ACTION_THIS_TURN}surveil").into(),
            ))
        }
        "you've scried this turn" => {
            return Some(Condition::Custom(format!("{ACTION_THIS_TURN}scry").into()))
        }
        _ => {}
    }
    let r = c
        .strip_prefix("you sacrificed ")
        .or_else(|| c.strip_prefix("you've sacrificed "))?;
    let r = r.strip_suffix(" this turn")?;
    let w = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let ty = match w {
        "permanent" => "permanent".to_string(),
        w => {
            let t = CardType::from_word(w)?;
            if !t.is_permanent_type() {
                return None;
            }
            w.to_string()
        }
    };
    Some(Condition::Custom(
        format!("{SACRIFICED_THIS_TURN}{ty}").into(),
    ))
}

fn owns_in_exile(c: &str) -> Option<Condition> {
    let (rel, r) = if let Some(r) = c.strip_prefix("an opponent owns ") {
        (PlayerRel::Opponent, r)
    } else if let Some(r) = c.strip_prefix("you own ") {
        (PlayerRel::You, r)
    } else {
        return None;
    };
    let desc = r.strip_suffix(" in exile")?;
    let f = match desc {
        "a card" => Filter::Card,
        d => {
            let d = d.strip_prefix("a ").or_else(|| d.strip_prefix("an "))?;
            let (f, plural, rest) = parse_object_phrase(d)?;
            if plural || !rest.trim().is_empty() {
                return None;
            }
            f
        }
    };
    Some(Condition::Exists(Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Exile),
        Filter::OwnedBy(rel),
    ])))
}

fn object_history(c: &str) -> Option<Condition> {
    match c {
        // A permanent's abilities see how it was cast (CR 607.2i, 601.2i).
        "it was cast" | "~ was cast" => return Some(Condition::WasCast),
        _ => {}
    }
    // "two or more creatures are blocking it"
    let r = c
        .strip_suffix(" are blocking it")
        .or_else(|| c.strip_suffix(" are blocking ~"))?;
    let (n, rest) = parse_number(r)?;
    let noun = strip(rest, "or more")?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if !plural || !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![f, Filter::BlockingSource])),
        Cmp::Ge,
        n,
    ))
}

fn grant_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    cast_this_turn(c)
        .or_else(|| history(c))
        .or_else(|| owns_in_exile(c))
        .or_else(|| object_history(c))
}

inventory::submit! { ConditionPattern { name: "grants: this turn's history, exile, blocking", priority: 120, parse: grant_condition } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_history_conditions() {
        for c in [
            "you've cast an instant or sorcery spell this turn",
            "you haven't cast a spell this turn",
            "you haven't cast a spell from your hand this turn",
            "you've committed a crime this turn",
            "you've surveilled this turn",
            "you sacrificed a permanent this turn",
            "you sacrificed an artifact this turn",
            "an opponent owns a card in exile",
            "it was cast",
            "two or more creatures are blocking it",
        ] {
            assert!(grant_condition(c).is_some(), "{c}");
        }
        for c in [
            "you've cast two creatures this turn",
            "you sacrificed an instant this turn",
            "an opponent owns cards in exile",
        ] {
            assert!(grant_condition(c).is_none(), "{c}");
        }
    }
}
