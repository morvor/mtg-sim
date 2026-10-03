//! CR 702.21 Ward.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Ward;

impl KeywordRules for Ward {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Ward]
    }

    /// CR 702.21b: X in a ward cost is determined as the ability resolves.
    fn x_determined_on_resolution(&self) -> bool {
        true
    }

    /// CR 702.21a: "Whenever this permanent becomes the target of a spell or ability an
    /// opponent controls, counter that spell or ability unless that player pays [cost]."
    /// CR 702.21b: when the cost has an X the ability defines, X is determined as the
    /// ability resolves, not when it triggers.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = match (&kw.x, kw.costs.first()) {
            // The cost with X, as granted ("ward {X}, where X is ..."); `cost` shows
            // its current value (CR 702.1b).
            (Some(_), Some(c)) => c.clone(),
            _ => kw.cost.clone().unwrap_or_default(),
        };
        let pay = Effect::PayOptional {
            who: PlayerRef::TriggerPlayer,
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::CounterSpell {
                what: Sel::TriggerSpell,
            }),
        };
        let effect = match &kw.x {
            Some(x) => Effect::Seq(vec![Effect::SetX { value: x.clone() }, pay]),
            None => pay,
        };
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BecomesTarget {
                    filter: Filter::Source,
                    by: PlayerRel::Opponent,
                },
                Body::effect(effect),
            )),
            KeywordKind::Ward.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Ward) }
