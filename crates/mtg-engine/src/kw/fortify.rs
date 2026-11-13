//! CR 702.67 Fortify.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Fortify;

impl KeywordRules for Fortify {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Fortify]
    }

    /// CR 702.67a: "Fortify [cost]" means "[Cost]: Attach this Fortification to target
    /// land you control. Activate only as a sorcery." (CR 702.67c: each instance is a
    /// separate ability.)
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = kw.cost.clone().unwrap_or_default();
        let mut act = ActivatedAbility::new(
            cost,
            Body::simple(
                vec![TargetSpec::object(
                    Filter::and(vec![
                        Filter::Type(crate::types::CardType::Land),
                        Filter::ControlledBy(PlayerRel::You),
                        Filter::Other,
                    ]),
                    "target land you control",
                )],
                Effect::Attach {
                    what: Sel::This,
                    to: Sel::Target(0),
                },
            ),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(AbilityKind::Activated(act), "Fortify")])
    }
}

inventory::submit! { KeywordRegistration(&Fortify) }
