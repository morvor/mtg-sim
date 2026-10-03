//! CR 702.163 For Mirrodin!: "When this Equipment enters, create a 2/2 red Rebel creature
//! token, then attach this Equipment to it." (CR 702.163a). If the Equipment has left the
//! battlefield by the time the ability resolves, the token is still created but nothing
//! is attached to it.
//!
//! Abilities that trigger on the token entering see it as it entered, a 2/2, before the
//! Equipment is attached to it (CR 603.2, 608.2c): triggers are checked between the two
//! instructions (see `trigger_timing`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The token For Mirrodin! creates: a 2/2 red Rebel creature token.
pub fn rebel() -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::single(Color::Red),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype::new("Rebel")],
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    }
}

pub struct ForMirrodin;

impl KeywordRules for ForMirrodin {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::ForMirrodin]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::effect(Effect::Seq(vec![
                    Effect::CreateToken {
                        spec: rebel(),
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
            KeywordKind::ForMirrodin.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&ForMirrodin) }
