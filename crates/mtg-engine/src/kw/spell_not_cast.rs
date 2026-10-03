//! "a spell that wasn't cast" (Errant, Street Artist: "Copy target spell you control that
//! wasn't cast."): a spell on the stack that no player cast — a copy of a spell, which is
//! put on the stack rather than cast (CR 707.10), or a card put onto the stack as a spell
//! without being cast. `Filter::Custom(NOT_CAST)`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom` name: a spell on the stack that wasn't cast.
pub const NOT_CAST: &str = "spell that wasn't cast";

pub struct SpellNotCast;

impl KeywordRules for SpellNotCast {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name != NOT_CAST {
            return None;
        }
        let o = g.obj(id);
        Some(o.is_spell() && o.stack.as_ref().is_some_and(|si| !si.cast.was_cast))
    }
}

inventory::submit! { KeywordRegistration(&SpellNotCast) }
