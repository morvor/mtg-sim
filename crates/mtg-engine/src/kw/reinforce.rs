//! CR 702.77 Reinforce.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::counters;
use smol_str::SmolStr;

pub struct Reinforce;

impl KeywordRules for Reinforce {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Reinforce]
    }

    /// CR 702.77a: "Reinforce N—[cost]" means "[Cost], Discard this card: Put N +1/+1
    /// counters on target creature." It can be activated only while the card is in a
    /// player's hand, but the ability exists in every zone (CR 702.77b), so a permanent with
    /// reinforce has an activated ability. "Reinforce X—{X}..." (N is -1) puts X counters.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = match kw.n {
            Some(-1) => Value::X,
            n => Value::c(n.unwrap_or(0)),
        };
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::DiscardSelf);
        let mut act = ActivatedAbility::new(
            cost,
            Body::simple(
                vec![TargetSpec::object(Filter::creature(), "target creature")],
                Effect::AddCounters {
                    what: Sel::Target(0),
                    kind: SmolStr::new(counters::PLUS1),
                    n,
                },
            ),
        );
        act.zone = FunctionZone::Hand;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Reinforce.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Reinforce) }
