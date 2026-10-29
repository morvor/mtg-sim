//! Conditions about a card in a graveyard and the cards above it (CR 404.1: a card is
//! above another if it was put into that graveyard later):
//!
//! * "~ is in your graveyard" (the ability functions from the graveyard, CR 113.6);
//! * "~ is in your graveyard with three or more creature cards above it";
//! * "~ is in your graveyard with a creature card directly above it";
//! * "three or more creature cards are above ~".

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number};
use crate::types::CardType;
use crate::zones::{ABOVE_SOURCE_IN_GRAVEYARD, DIRECTLY_ABOVE_SOURCE_IN_GRAVEYARD, STILL_THERE};
use smol_str::SmolStr;

/// The source is (still) in its owner's graveyard.
fn source_in_your_graveyard() -> Condition {
    Condition::SelMatches(
        Sel::This,
        Filter::And(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
            Filter::Custom(SmolStr::new(STILL_THERE)),
        ]),
    )
}

/// Creature cards above the source in its graveyard (directly above: the one just above).
fn creature_cards_above(directly: bool) -> Filter {
    let above = if directly {
        DIRECTLY_ABOVE_SOURCE_IN_GRAVEYARD
    } else {
        ABOVE_SOURCE_IN_GRAVEYARD
    };
    Filter::And(vec![
        Filter::Card,
        Filter::Type(CardType::Creature),
        Filter::InZone(ZoneKind::Graveyard),
        Filter::Custom(SmolStr::new(above)),
    ])
}

/// "three or more creature cards" → at least 3.
fn at_least_creature_cards(s: &str) -> Option<Condition> {
    let (n, r) = parse_number(s)?;
    n.as_const()?;
    let r = r.strip_prefix("or more creature cards")?;
    r.is_empty().then(|| {
        Condition::Compare(
            Value::Count(creature_cards_above(false)),
            Cmp::Ge,
            n,
        )
    })
}

fn graveyard_order_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    if c == "~ is in your graveyard" {
        return Some(source_in_your_graveyard());
    }
    if let Some(r) = c.strip_prefix("~ is in your graveyard with ") {
        let above = if r == "a creature card directly above it" {
            Condition::Exists(creature_cards_above(true))
        } else {
            at_least_creature_cards(r.strip_suffix(" above it")?)?
        };
        return Some(Condition::And(vec![source_in_your_graveyard(), above]));
    }
    // "three or more creature cards are above ~" (while ~ is in a graveyard).
    at_least_creature_cards(c.strip_suffix(" are above ~")?)
}

inventory::submit! { ConditionPattern { name: "graveyard order: cards above ~", priority: 0, parse: graveyard_order_condition } }

/// Whether a condition requires the source to be in a graveyard: an ability with it (an
/// intervening "if ~ is in your graveyard" clause) functions from the graveyard
/// (CR 113.6).
pub(crate) fn requires_source_in_graveyard(c: &Condition) -> bool {
    match c {
        Condition::SelMatches(Sel::This, Filter::And(fs)) => fs
            .iter()
            .any(|f| matches!(f, Filter::InZone(ZoneKind::Graveyard))),
        Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Graveyard)) => true,
        Condition::And(cs) => cs.iter().any(requires_source_in_graveyard),
        _ => false,
    }
}
