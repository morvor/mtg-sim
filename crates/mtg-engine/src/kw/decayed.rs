//! CR 702.147 Decayed: "Decayed" means "This creature can't block" and "When this
//! creature attacks, sacrifice it at end of combat." (CR 702.147a).
//!
//! The triggered ability creates a delayed triggered ability (CR 603.7) as it resolves:
//! at the end of combat, the creature is sacrificed if it's still on the battlefield and
//! its controller (the ability's controller) still controls it (CR 701.21a). It doesn't
//! matter whether it still has decayed then.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Decayed;

impl KeywordRules for Decayed {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Decayed]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Decayed.name();
        let cant_block =
            StaticAbility::new(StaticEffect::Restriction(Restriction::CantBlock(Filter::Source)));
        let sacrifice = TriggeredAbility::new(
            TriggerCond::Attacks(Filter::Source),
            Body::effect(Effect::AtNext {
                step: TriggerStep::EndOfCombat,
                // "Sacrifice it": only its controller can sacrifice it (CR 701.21a); the
                // same object, if it's still on the battlefield (CR 400.7).
                effect: Box::new(Effect::If {
                    cond: Condition::SelMatches(Sel::This, Filter::ControlledBy(PlayerRel::You)),
                    then: Box::new(Effect::SacrificeObjects { what: Sel::This }),
                    otherwise: Box::new(Effect::Noop),
                }),
            }),
        );
        Some(vec![
            AbilityDef::new(AbilityKind::Static(cant_block), text),
            AbilityDef::new(AbilityKind::Triggered(sacrifice), text),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Decayed) }
