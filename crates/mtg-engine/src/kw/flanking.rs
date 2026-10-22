//! CR 702.25 Flanking.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Flanking;

impl KeywordRules for Flanking {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Flanking]
    }

    /// CR 702.25a: "Whenever this creature becomes blocked by a creature without
    /// flanking, the blocking creature gets -1/-1 until end of turn." It triggers once
    /// per such blocker (CR 509.3d), and each instance triggers separately (CR 702.25b).
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BlockedByCreature {
                    attacker: Filter::Source,
                    blocker: Filter::not(Filter::HasKeyword(KeywordKind::Flanking)),
                },
                Body::effect(Effect::Modify {
                    what: Sel::TriggerObject,
                    mods: vec![Modification::ModifyPT(Value::c(-1), Value::c(-1))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            KeywordKind::Flanking.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Flanking) }
