//! CR 702.182 Job select: "When this Equipment enters, create a 1/1 colorless Hero
//! creature token, then attach this Equipment to it." (CR 702.182a).
//!
//! If the Equipment has left the battlefield by the time the ability resolves, the token
//! is still created but nothing is attached to it. If a replacement effect creates more
//! than one token, the Equipment is attached to only one of them.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The token job select creates: a 1/1 colorless Hero creature token.
pub fn hero() -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype::new("Hero")],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
    }
}

pub struct JobSelect;

impl KeywordRules for JobSelect {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::JobSelect]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::effect(Effect::Seq(vec![
                    Effect::CreateToken {
                        spec: hero(),
                        count: Value::c(1),
                        controller: PlayerRef::You,
                        tapped: false,
                        attacking: false,
                    },
                    Effect::Attach {
                        what: Sel::This,
                        to: Sel::Var(vars::CREATED),
                    },
                ])),
            )),
            KeywordKind::JobSelect.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&JobSelect) }
