//! `Condition::Custom` [`HAS_ANOTHER_ACTIVATED_ABILITY`]: "Activate only if ~ has another
//! activated ability." (the mana ability Tazri, Stalwart Survivor grants): the source has
//! an activated ability besides this one. Any activated ability counts, including one
//! that can't be activated at the moment or while the creature is on the battlefield, such
//! as cycling (Tazri's ruling), and mana abilities (CR 602.1, 605.1a). Keyword abilities
//! that are activated abilities count as the abilities they stand for.

use crate::ability::AbilityKind;
use crate::eval::Ctx;
use crate::game::Game;

/// The `Condition::Custom` name.
pub const HAS_ANOTHER_ACTIVATED_ABILITY: &str = "source has another activated ability";

pub struct HasAnotherActivatedAbility;

impl super::KeywordRules for HasAnotherActivatedAbility {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != HAS_ANOTHER_ACTIVATED_ABILITY {
            return None;
        }
        let Some(s) = ctx.source.and_then(|s| g.try_obj(s)) else {
            return Some(false);
        };
        // This ability is one of the source's activated abilities: another one makes two.
        let n = s
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .count();
        Some(n >= 2)
    }
}

inventory::submit! { super::KeywordRegistration(&HasAnotherActivatedAbility) }
