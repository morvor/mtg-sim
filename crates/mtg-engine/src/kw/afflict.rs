//! CR 702.130 Afflict: "Afflict N" means "Whenever this creature becomes blocked,
//! defending player loses N life." (CR 702.130a). Each instance triggers separately
//! (CR 702.130b): each is its own triggered ability.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Afflict;

impl KeywordRules for Afflict {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Afflict]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BecomesBlocked(Filter::Source),
                Body::effect(Effect::LoseLife {
                    who: PlayerRef::DefendingPlayer,
                    n: Value::c(n),
                }),
            )),
            format!("Afflict {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Afflict) }
