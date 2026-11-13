//! CR 702.91 Battle cry.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct BattleCry;

impl KeywordRules for BattleCry {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::BattleCry]
    }

    /// CR 702.91a: "Battle cry" means "Whenever this creature attacks, each other
    /// attacking creature gets +1/+0 until end of turn." The creatures affected are
    /// determined as the ability resolves (CR 611.2c). Each instance triggers separately
    /// (CR 702.91b).
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::effect(Effect::Modify {
                    what: Sel::All(Filter::and(vec![
                        Filter::creature(),
                        Filter::Attacking,
                        Filter::Other,
                    ])),
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(0))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            KeywordKind::BattleCry.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&BattleCry) }
