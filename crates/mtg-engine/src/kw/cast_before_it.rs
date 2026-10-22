//! "[spells] you've cast before it this turn" (Thousand-Year Storm): the spells cast this
//! turn earlier than the spell whose casting triggered the ability — not the spell itself,
//! and not spells cast after it (in response to the trigger). Copies of spells were never
//! cast (CR 707.10) and aren't counted.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: a spell cast this turn before the spell that triggered the ability.
pub const CAST_BEFORE_IT: &str = "cast before it:cast this turn before the triggering spell";

pub struct CastBeforeIt;

impl KeywordRules for CastBeforeIt {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != CAST_BEFORE_IT {
            return None;
        }
        let Some(spell) = ctx.event.as_ref().and_then(|e| e.spell) else {
            return Some(false);
        };
        let pos = |x: ObjectId| g.history.spells_cast.iter().position(|(_, s)| *s == x);
        Some(matches!((pos(id), pos(spell)), (Some(a), Some(b)) if a < b))
    }
}

inventory::submit! { KeywordRegistration(&CastBeforeIt) }
