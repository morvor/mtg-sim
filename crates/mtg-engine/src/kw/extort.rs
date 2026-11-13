//! CR 702.101 Extort: "Whenever you cast a spell, you may pay {W/B}. If you do, each
//! opponent loses 1 life and you gain life equal to the total life lost this way."
//! (CR 702.101a). Each instance triggers separately (CR 702.101b). The payment is chosen
//! as the ability resolves; the ability doesn't target.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;

pub struct Extort;

impl KeywordRules for Extort {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Extort]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = Cost::mana(ManaCost::parse("{W/B}").unwrap_or_default());
        let t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Any,
            },
            Body::effect(Effect::PayOptional {
                who: PlayerRef::You,
                cost,
                then: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: PlayerRef::EachOpponent,
                        n: Value::c(1),
                    },
                    // The total life actually lost (e.g. not by a player whose life total
                    // can't change).
                    Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::Prev,
                    },
                ])),
                otherwise: Box::new(Effect::Noop),
            }),
        );
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Extort.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Extort) }
