//! CR 702.108 Prowess: "Whenever you cast a noncreature spell, this creature gets +1/+1
//! until end of turn." (CR 702.108a). Each instance triggers separately (CR 702.108b):
//! every instance of a keyword on an object gets its own derived ability (see
//! `keyword_impls::expand_keywords`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Prowess;

impl KeywordRules for Prowess {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Prowess]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        // Once it triggers it isn't connected to the spell: it resolves (first, being on
        // top of it) even if that spell is countered.
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::CastSpell {
                    who: PlayerRel::You,
                    filter: Filter::not(Filter::creature()),
                },
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            KeywordKind::Prowess.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Prowess) }
