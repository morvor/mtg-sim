//! CR 702.70 Poisonous.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::counters;
use smol_str::SmolStr;

pub struct Poisonous;

impl KeywordRules for Poisonous {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Poisonous]
    }

    /// CR 702.70a: "Poisonous N" means "Whenever this creature deals combat damage to a
    /// player, that player gets N poison counters." However much damage it dealt, the
    /// player gets N counters. Each instance triggers separately (CR 702.70b).
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::DealsDamage {
                    source: Filter::Source,
                    to: DamageRecipient::Player(PlayerRel::Any),
                    combat_only: true,
                },
                Body::effect(Effect::AddPlayerCounters {
                    who: PlayerRef::TriggerPlayer,
                    kind: SmolStr::new(counters::POISON),
                    n: Value::c(n),
                }),
            )),
            format!("Poisonous {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Poisonous) }
