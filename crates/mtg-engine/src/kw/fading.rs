//! CR 702.32 Fading: "Fading N" means "This permanent enters with N fade counters on it"
//! and "At the beginning of your upkeep, remove a fade counter from this permanent. If
//! you can't, sacrifice the permanent." (CR 702.32a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

pub struct Fading;

fn fade_counters() -> Value {
    Value::CountersOn(Box::new(Sel::This), Some(counters::FADE.into()))
}

impl KeywordRules for Fading {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Fading]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        let text = KeywordKind::Fading.name();
        let enters = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::EnterWithCounters(counters::FADE.into(), Value::c(n)),
            self_replacement: false,
            optional: false,
        }));
        let upkeep = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::If {
                cond: Condition::Compare(fade_counters(), Cmp::Gt, Value::c(0)),
                then: Box::new(Effect::RemoveCounters {
                    what: Sel::This,
                    kind: Some(counters::FADE.into()),
                    n: Value::c(1),
                }),
                otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
            }),
        );
        Some(vec![
            AbilityDef::new(AbilityKind::Static(enters), text),
            AbilityDef::new(AbilityKind::Triggered(upkeep), text),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Fading) }
