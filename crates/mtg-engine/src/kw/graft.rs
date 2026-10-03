//! CR 702.58 Graft. "Graft N" means "This permanent enters with N +1/+1 counters on it"
//! and "Whenever another creature enters, if this permanent has a +1/+1 counter on it,
//! you may move a +1/+1 counter from this permanent onto that creature." (CR 702.58a).
//! Each instance works separately (CR 702.58b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

pub struct Graft;

impl KeywordRules for Graft {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Graft]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Graft.name();
        let n = kw.n.unwrap_or(0).max(0);
        let enters = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::EnterWithCounters(counters::PLUS1.into(), Value::c(n)),
            self_replacement: false,
            optional: false,
        }));
        let mut moves = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::creature().other()),
            Body::effect(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::MoveCounters {
                    from: Sel::This,
                    to: Sel::TriggerObject,
                    kind: Some(counters::PLUS1.into()),
                    n: Some(Value::c(1)),
                }),
            }),
        );
        moves.intervening_if = Some(Condition::Compare(
            Value::CountersOn(Box::new(Sel::This), Some(counters::PLUS1.into())),
            Cmp::Gt,
            Value::c(0),
        ));
        Some(vec![
            AbilityDef::new(AbilityKind::Static(enters), text),
            AbilityDef::new(AbilityKind::Triggered(moves), text),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Graft) }
