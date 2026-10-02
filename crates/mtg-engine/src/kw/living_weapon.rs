//! CR 702.92 Living weapon.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

pub struct LivingWeapon;

/// `Effect::Custom`: the Germ's entering is checked for triggers as it happens, before the
/// Equipment is attached to it (CR 603.2): abilities that trigger on it entering see a 0/0
/// creature.
const GERM_ENTERED: &str = "living weapon:the germ entered";

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
        pt_values: None,
    }
}

impl KeywordRules for LivingWeapon {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::LivingWeapon]
    }

    /// CR 702.92a: "Living weapon" means "When this Equipment enters, create a 0/0 black
    /// Phyrexian Germ creature token, then attach this Equipment to it." If the Equipment
    /// has left the battlefield by then, the token is still created but nothing is
    /// attached to it. The Germ enters as a 0/0 creature, and triggers see it that way.
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
                    Effect::Custom(GERM_ENTERED.into()),
                    Effect::Attach {
                        what: Sel::This,
                        to: Sel::Var(vars::CREATED),
                    },
                ])),
            )),
            KeywordKind::LivingWeapon.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        if name != GERM_ENTERED {
            return false;
        }
        g.flush_events();
        true
    }
}

inventory::submit! { KeywordRegistration(&LivingWeapon) }
