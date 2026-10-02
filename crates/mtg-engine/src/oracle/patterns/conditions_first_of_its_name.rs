//! Intervening "if" conditions (CR 603.4) about the triggering object's name: "if it
//! doesn't have the same name as another creature you control or a creature card in
//! your graveyard" (Guardian Project). "It" is the object the trigger event is about; a
//! new object it became (CR 400.7) — the card in your graveyard, or a permanent that
//! returned to the battlefield — is another object with that name.

use super::ConditionPattern;
use crate::ability::*;
use crate::types::CardType;

fn not_first_of_its_name(c: &str) -> Option<Condition> {
    let r = crate::oracle::phrases::end(c).strip_prefix("it doesn't have the same name as ")?;
    let same = || Filter::SameNameAs(Box::new(Sel::TriggerObject));
    let mut found = Vec::new();
    for part in r.split(" or ") {
        found.push(match part {
            "another creature you control" => Condition::Exists(Filter::And(vec![
                Filter::Type(CardType::Creature),
                Filter::ControlledBy(PlayerRel::You),
                same(),
                Filter::not(Filter::In(Box::new(Sel::TriggerObject))),
            ])),
            "a creature card in your graveyard" => Condition::Exists(Filter::And(vec![
                Filter::Type(CardType::Creature),
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
                same(),
            ])),
            _ => return None,
        });
    }
    Some(Condition::Not(Box::new(Condition::Or(found))))
}

inventory::submit! { ConditionPattern { name: "it doesn't have the same name as another [creature you control / card in your graveyard]", priority: 100, parse: not_first_of_its_name } }
