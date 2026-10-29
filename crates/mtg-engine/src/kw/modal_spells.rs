//! Modal spells (CR 700.2) as a quality of a spell and the number of modes chosen for one:
//! "Whenever you cast a modal spell, choose up to X, where X is the number of times you
//! chose a mode for that spell —" (Riku of Many Paths).
//!
//! * A spell is modal if its spell ability has modes to choose from (CR 700.2), including
//!   spree and tiered spells (each mode with an additional cost, CR 700.2h) and pawprint
//!   spells (CR 700.2i). A copy of a modal spell is modal too (CR 707.2).
//! * The number of times a mode was chosen counts a mode chosen more than once once per
//!   choice (CR 700.2d), from the spell's last known information if it has left the stack
//!   (CR 608.2h).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::AbilityKind;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom` name: a modal spell (CR 700.2).
pub const MODAL_SPELL: &str = "modal:modal spell";

/// `Value::Custom` name: the number of times a mode was chosen for the spell that caused
/// the ability to trigger ("that spell").
pub const MODES_CHOSEN_FOR_THAT_SPELL: &str = "modal:times a mode was chosen for that spell";

/// Whether `id` is a modal spell (or card): one of its spell abilities has modes.
pub fn is_modal(g: &Game, id: ObjectId) -> bool {
    g.obj(id).chars.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Spell(s) => s.body.modal.is_some(),
        _ => false,
    })
}

/// The number of modes chosen for the triggering spell of `ctx` (each choice of a mode
/// counted, CR 700.2d); 0 if there's no such spell.
pub fn modes_chosen_for_that_spell(g: &Game, ctx: &Ctx) -> i64 {
    let Some(spell) = ctx.event.as_ref().and_then(|e| e.spell.or(e.object)) else {
        return 0;
    };
    g.obj(spell).stack.as_deref().map_or(0, |si| {
        si.chosen.iter().filter(|m| m.mode.is_some()).count() as i64
    })
}

pub struct ModalSpells;

impl KeywordRules for ModalSpells {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == MODES_CHOSEN_FOR_THAT_SPELL).then(|| modes_chosen_for_that_spell(g, ctx))
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == MODAL_SPELL).then(|| is_modal(g, id))
    }
}

inventory::submit! { KeywordRegistration(&ModalSpells) }
