//! CR 702.97 Scavenge.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::counters;

pub struct Scavenge;

impl KeywordRules for Scavenge {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Scavenge]
    }

    /// CR 702.97a: "Scavenge [cost]" means "[Cost], Exile this card from your graveyard:
    /// Put a number of +1/+1 counters equal to the power of the card you exiled on target
    /// creature. Activate only as a sorcery." The ability functions only while the card is
    /// in a graveyard; the power is that of the card as it last existed in the graveyard
    /// (its last known information, CR 608.2h).
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::ExileSelf);
        let mut act = ActivatedAbility::new(
            cost,
            Body::simple(
                vec![TargetSpec::object(Filter::creature(), "target creature")],
                Effect::AddCounters {
                    what: Sel::Target(0),
                    kind: counters::PLUS1.into(),
                    n: Value::PowerOf(Box::new(Sel::This)),
                },
            ),
        );
        act.timing = ActivationTiming::Sorcery;
        act.zone = FunctionZone::Graveyard;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Scavenge.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Scavenge) }
