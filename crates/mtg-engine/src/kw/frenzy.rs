//! CR 702.68 Frenzy.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Frenzy;

impl KeywordRules for Frenzy {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Frenzy]
    }

    /// CR 702.68a: "Frenzy N" means "Whenever this creature attacks and isn't blocked, it
    /// gets +N/+0 until end of turn." Each instance triggers separately (CR 702.68b).
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::AttacksUnblocked(Filter::Source),
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(Value::c(n), Value::c(0))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            format!("Frenzy {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Frenzy) }
