//! CR 702.92 Living weapon.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

pub struct LivingWeapon;

/// The token living weapon creates: a 0/0 black Phyrexian Germ creature token.
pub fn germ() -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::single(Color::Black),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype::new("Phyrexian"), Subtype::new("Germ")],
        power: Some(0),
        toughness: Some(0),
        abilities: vec![],
        scryfall_name: None,
    }
}

impl KeywordRules for LivingWeapon {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::LivingWeapon]
    }

    /// CR 702.92a: "Living weapon" means "When this Equipment enters, create a 0/0 black
    /// Phyrexian Germ creature token, then attach this Equipment to it." If the Equipment
    /// has left the battlefield by then, the token is still created but nothing is
    /// attached to it.
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::effect(Effect::Seq(vec![
                    Effect::CreateToken {
                        spec: germ(),
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
            KeywordKind::LivingWeapon.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&LivingWeapon) }
