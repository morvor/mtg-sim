//! Loyalty abilities on the stack (CR 606): "Whenever you activate a loyalty ability of a
//! Chandra planeswalker" (Chandra's Regulator) qualifies the activation by the kind of
//! ability activated.

use crate::ability::AbilityKind;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::StackKind;
use crate::types::ObjectId;

/// `Filter::Custom` name: an activated ability on the stack that is a loyalty ability
/// (CR 606.3).
pub const LOYALTY_ABILITY: &str = "loyalty ability";

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
    if name != LOYALTY_ABILITY {
        return None;
    }
    Some(matches!(
        g.obj(id).stack.as_deref().map(|si| &si.kind),
        Some(StackKind::Activated { ability, .. })
            if matches!(&ability.kind, AbilityKind::Activated(a) if a.is_loyalty)
    ))
}
