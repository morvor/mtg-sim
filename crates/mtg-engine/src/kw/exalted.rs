//! CR 702.83 Exalted.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Exalted;

impl KeywordRules for Exalted {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Exalted]
    }

    /// CR 702.83a: "Exalted" means "Whenever a creature you control attacks alone, that
    /// creature gets +1/+1 until end of turn." A creature attacks alone if it's the only
    /// creature declared as an attacker in a given combat phase (CR 702.83b, 506.5): it
    /// triggers as attackers are declared. Each instance (on any permanent, including the
    /// attacking creature) triggers separately and pumps the attacking creature.
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::AttacksAlone(Filter::and(vec![
                    Filter::creature(),
                    Filter::ControlledBy(PlayerRel::You),
                ])),
                Body::effect(Effect::Modify {
                    what: Sel::TriggerObject,
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            KeywordKind::Exalted.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Exalted) }
