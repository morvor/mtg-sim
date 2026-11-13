//! Trigger events and conditions of old (paper) card wordings:
//!
//! * "the chosen color is the most common color among nontoken permanents the chosen
//!   player controls but isn't tied for most common" and its negation "... isn't the most
//!   common color ... or is tied for most common" (Call to Arms; see
//!   `kw::most_common_chosen_color`), also as a state trigger (CR 603.8);
//! * "When ~'s power is 7 or greater, sacrifice it." (Phyrexian Devourer): a power or
//!   toughness condition as a state trigger (CR 603.8; its ruling: once it triggers, it's
//!   sacrificed even if its power has dropped by then);
//! * "When you play another land, sacrifice ~." (City of Traitors): playing a land other
//!   than this one (its rulings: lands put onto the battlefield without being played don't
//!   count, nor does playing ~ itself);
//! * "Whenever a Swamp, Mountain, black permanent, or red permanent becomes tapped, ..."
//!   (Royal Decree): a list of kinds of permanents as the subject, any of which (its
//!   ruling: one trigger per permanent, even one matching several kinds).

use super::{ConditionPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::most_common_chosen_color as mc;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

type Parsed = (TriggerCond, Sel, PlayerRef);

/// "[the chosen color / white] is the most common color among nontoken permanents the
/// chosen player controls but isn't tied for most common", "[...] isn't the most common
/// color among ... or is tied for most common".
fn most_common_chosen(c: &str) -> Option<Condition> {
    let c = end(c.trim());
    let (color, rest, negated) = if let Some((a, b)) =
        c.split_once(" isn't the most common color among ")
    {
        (a, b, true)
    } else {
        let (a, b) = c.split_once(" is the most common color among ")?;
        (a, b, false)
    };
    let color = match color {
        "the chosen color" => None,
        w => Some(crate::types::Color::from_word(w)?),
    };
    let (among, tie) = rest.split_once(" controls ")?;
    let among = match among {
        "nontoken permanents the chosen player" => mc::Among::ChosenPlayersNontoken,
        _ => return None,
    };
    // "is X but isn't tied" holds; "isn't X or is tied" is its negation.
    match (negated, tie) {
        (false, "but isn't tied for most common") => {
            Some(Condition::Custom(mc::name(color, among, true).into()))
        }
        (true, "or is tied for most common") => Some(Condition::Not(Box::new(
            Condition::Custom(mc::name(color, among, true).into()),
        ))),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "paper triggers: the chosen color is the most common color among nontoken permanents the chosen player controls", priority: 100, parse: most_common_chosen } }

/// "When the chosen color isn't the most common color among ... or is tied for most
/// common" (CR 603.8).
fn most_common_state_trigger(r: &str) -> Option<Parsed> {
    let r = end(r).strip_prefix("when ").unwrap_or(end(r));
    if !r.starts_with("the chosen color ") {
        return None;
    }
    Some((TriggerCond::State(most_common_chosen(r)?), Sel::This, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "paper triggers: when the chosen color isn't the most common color ...", priority: 100, parse: most_common_state_trigger } }

/// "When ~'s power is 7 or greater", "When ~'s toughness is 2 or less" (CR 603.8).
fn stat_state_trigger(r: &str) -> Option<Parsed> {
    let r = end(r).strip_prefix("when ").unwrap_or(end(r));
    let (stat, rest) = if let Some(x) = r.strip_prefix("~'s power is ") {
        (Value::PowerOf(Box::new(Sel::This)), x)
    } else {
        (
            Value::ToughnessOf(Box::new(Sel::This)),
            r.strip_prefix("~'s toughness is ")?,
        )
    };
    let (n, tail) = parse_number(rest)?;
    n.as_const()?;
    let cmp = match tail.trim() {
        "or greater" => Cmp::Ge,
        "or less" => Cmp::Le,
        _ => return None,
    };
    Some((
        TriggerCond::State(Condition::Compare(stat, cmp, n)),
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "paper triggers: when ~'s power is N or greater", priority: 100, parse: stat_state_trigger } }

/// "When you play another land" (City of Traitors).
fn play_another_land(r: &str) -> Option<Parsed> {
    let r = end(r);
    let r = r
        .strip_prefix("when ")
        .or_else(|| r.strip_prefix("whenever "))
        .unwrap_or(r);
    if r != "you play another land" {
        return None;
    }
    crate::oracle::triggers::parse_trigger_condition("whenever you play a land other than ~")
}

inventory::submit! { TriggerPattern { name: "paper triggers: when you play another land", priority: 100, parse: play_another_land } }

/// "Whenever a Swamp, Mountain, black permanent, or red permanent becomes tapped".
fn list_becomes_tapped(r: &str) -> Option<Parsed> {
    let r = end(r);
    let r = r.strip_prefix("whenever ").unwrap_or(r);
    let list = r.strip_suffix(" becomes tapped")?;
    let list = list.strip_prefix("a ").or_else(|| list.strip_prefix("an "))?;
    let (head, last) = list.rsplit_once(", or ")?;
    let mut items: Vec<&str> = head.split(", ").collect();
    items.push(last);
    if items.len() < 3 {
        return None;
    }
    let mut kinds = Vec::new();
    for item in items {
        let (f, plural, tail) = parse_object_phrase(item)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        kinds.push(Filter::and(vec![f, Filter::Permanent]));
    }
    let (cond, it, player) =
        crate::oracle::triggers::parse_trigger_condition("whenever a permanent becomes tapped")?;
    let TriggerCond::BecomesTapped(_) = cond else {
        return None;
    };
    Some((TriggerCond::BecomesTapped(Filter::Or(kinds)), it, player))
}

inventory::submit! { TriggerPattern { name: "paper triggers: whenever a [kind], [kind], or [kind] becomes tapped", priority: 100, parse: list_becomes_tapped } }
