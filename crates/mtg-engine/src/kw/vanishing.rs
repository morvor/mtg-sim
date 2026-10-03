//! CR 702.63 Vanishing. "Vanishing N" means "This permanent enters with N time counters
//! on it," "At the beginning of your upkeep, if this permanent has a time counter on it,
//! remove a time counter from it," and "When the last time counter is removed from this
//! permanent, sacrifice it." (CR 702.63a). Vanishing without a number stands for only
//! the last two abilities (CR 702.63b). Each instance works separately (CR 702.63c): each
//! adds its own counters and has its own triggered abilities.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

pub struct Vanishing;

fn time_counters() -> Value {
    Value::CountersOn(Box::new(Sel::This), Some(counters::TIME.into()))
}

impl KeywordRules for Vanishing {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Vanishing]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Vanishing.name();
        let mut out = Vec::new();
        // CR 702.63b: without a number, it doesn't add counters as it enters.
        if let Some(n) = kw.n.filter(|n| *n > 0) {
            let enters = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::EnterWithCounters(counters::TIME.into(), Value::c(n)),
                self_replacement: false,
                optional: false,
            }));
            out.push(AbilityDef::new(AbilityKind::Static(enters), text));
        }
        let mut upkeep = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::RemoveCounters {
                what: Sel::This,
                kind: Some(counters::TIME.into()),
                n: Value::c(1),
            }),
        );
        upkeep.intervening_if = Some(Condition::Compare(time_counters(), Cmp::Gt, Value::c(0)));
        // "When the last time counter is removed": however it was removed.
        let last = TriggeredAbility::new(
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::CountersRemoved {
                    filter: Filter::Source,
                    kind: Some(counters::TIME.into()),
                }),
                cond: Condition::Compare(time_counters(), Cmp::Eq, Value::c(0)),
            },
            Body::effect(Effect::SacrificeObjects { what: Sel::This }),
        );
        out.push(AbilityDef::new(AbilityKind::Triggered(upkeep), text));
        out.push(AbilityDef::new(AbilityKind::Triggered(last), text));
        Some(out)
    }
}

inventory::submit! { KeywordRegistration(&Vanishing) }
