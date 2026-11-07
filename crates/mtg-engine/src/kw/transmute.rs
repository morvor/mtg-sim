//! CR 702.53 Transmute. "Transmute [cost]" means "[Cost], Discard this card: Search your
//! library for a card with the same mana value as the discarded card, reveal that card,
//! and put it into your hand. Then shuffle your library. Activate only as a sorcery."
//! (CR 702.53a). The ability functions only while the card is in a player's hand, but it
//! exists in every zone (CR 702.53b): the card on the battlefield still has an activated
//! ability.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Transmute;

impl KeywordRules for Transmute {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Transmute]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::DiscardSelf);
        // "the discarded card": the card as it last existed in the hand.
        let effect = Effect::Search {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            filter: Filter::ManaValue(Cmp::Eq, Box::new(Value::ManaValueOf(Box::new(Sel::This)))),
            count: Value::c(1),
            to: Destination::zone(ZoneKind::Hand),
            reveal: true,
            shuffle: true,
        };
        let mut act = ActivatedAbility::new(cost, Body::effect(effect));
        act.zone = FunctionZone::Hand;
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Transmute.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Transmute) }
