//! CR 702.71 Transfigure.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Transfigure;

impl KeywordRules for Transfigure {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Transfigure]
    }

    /// CR 702.71a: "Transfigure [cost]" means "[Cost], Sacrifice this permanent: Search
    /// your library for a creature card with the same mana value as this permanent and put
    /// it onto the battlefield. Then shuffle your library. Activate only as a sorcery."
    /// "This permanent" is the sacrificed permanent as it last existed on the battlefield.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::SacrificeSelf);
        let effect = Effect::Search {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::creature(),
                Filter::ManaValue(Cmp::Eq, Box::new(Value::ManaValueOf(Box::new(Sel::This)))),
            ]),
            count: Value::c(1),
            to: Destination::battlefield(),
            reveal: false,
            shuffle: true,
        };
        let mut act = ActivatedAbility::new(cost, Body::effect(effect));
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Transfigure.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Transfigure) }
