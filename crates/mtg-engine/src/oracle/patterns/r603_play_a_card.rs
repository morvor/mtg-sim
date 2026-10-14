//! "Whenever you play a card, [effect]" (Recycle, Null Profusion, Juju Bubble): to play a
//! card is to play that card as a land or cast that card as a spell (glossary "Play"), so
//! the ability triggers when you cast a card (not a copy) or play a land. It triggers on
//! casting (CR 601.2i) and resolves even if the spell is countered. "Whenever you play a
//! card from exile" (Prosper, Tome-Bound): casting a card from exile, or playing a land
//! from exile.

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn you_play_a_card(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if end(r) != "you play a card" {
        return None;
    }
    Some((
        TriggerCond::AnyOf(vec![
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Card,
            },
            TriggerCond::LandPlayed {
                who: PlayerRel::You,
                filter: Filter::Any,
            },
        ]),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you play a card", priority: 100, parse: you_play_a_card } }

fn you_play_a_card_from_exile(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if end(r) != "you play a card from exile" {
        return None;
    }
    Some((
        TriggerCond::AnyOf(vec![
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::and(vec![Filter::Card, Filter::CastFrom(ZoneKind::Exile)]),
            },
            TriggerCond::LandPlayed {
                who: PlayerRel::You,
                filter: Filter::Custom(
                    crate::kw::played_from_zone::came_from(ZoneKind::Exile).into(),
                ),
            },
        ]),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you play a card from exile", priority: 100, parse: you_play_a_card_from_exile } }
