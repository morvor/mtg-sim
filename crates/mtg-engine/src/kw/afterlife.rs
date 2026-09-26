//! CR 702.135 Afterlife: "Afterlife N" means "When this permanent is put into a graveyard
//! from the battlefield, create N 1/1 white and black Spirit creature tokens with flying."
//! (CR 702.135a). Each instance triggers separately (CR 702.135b). The ability's
//! controller creates the tokens: the controller of the permanent as it last existed on
//! the battlefield (CR 603.3a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

pub struct Afterlife;

/// The 1/1 white and black Spirit creature token with flying afterlife creates.
pub fn spirit() -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::single(Color::White).union(ColorSet::single(Color::Black)),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype::new("Spirit")],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDef::new(
            AbilityKind::Keyword(Keyword::new(KeywordKind::Flying)),
            "Flying",
        )],
        scryfall_name: None,
    }
}

impl KeywordRules for Afterlife {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Afterlife]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(1).max(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Dies(Filter::Source),
                Body::effect(Effect::CreateToken {
                    spec: spirit(),
                    count: Value::c(n),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                }),
            )),
            format!("Afterlife {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Afterlife) }
