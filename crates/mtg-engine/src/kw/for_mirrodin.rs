//! CR 702.163 For Mirrodin!: "When this Equipment enters, create a 2/2 red Rebel creature
//! token, then attach this Equipment to it." (CR 702.163a). If the Equipment has left the
//! battlefield by the time the ability resolves, the token is still created but nothing
//! is attached to it.
//!
//! Abilities that trigger on the token entering see it as it entered, a 2/2, before the
//! Equipment is attached to it (CR 603.2): triggers are checked between the two steps.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: "create a 2/2 red Rebel creature token, then attach this Equipment to
/// it".
pub const REBEL_THEN_ATTACH: &str = "for mirrodin:create a Rebel, then attach this to it";

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
                Body::effect(Effect::Custom(REBEL_THEN_ATTACH.into())),
            )),
            KeywordKind::ForMirrodin.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REBEL_THEN_ATTACH {
            return false;
        }
        g.exec(
            &Effect::CreateToken {
                spec: rebel(),
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
            },
            ctx,
        );
        // CR 603.2: the token's entering is checked for triggers as it happens.
        g.flush_events();
        g.exec(
            &Effect::Attach {
                what: Sel::This,
                to: Sel::Var(vars::CREATED),
            },
            ctx,
        );
        true
    }
}

inventory::submit! { KeywordRegistration(&ForMirrodin) }
