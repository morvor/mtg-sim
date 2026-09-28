//! Descriptions of activated abilities on the stack, for triggers qualified by the kind of
//! ability activated: "Whenever you activate a loyalty ability of a Chandra planeswalker"
//! (Chandra's Regulator, CR 606), "When you next ... activate an ability with {X} in its
//! activation cost this turn" (Magus Lucea Kane, CR 107.3).

use crate::ability::{AbilityKind, ActivatedAbility};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::StackKind;
use crate::types::ObjectId;

/// `Filter::Custom` name: an activated ability on the stack that is a loyalty ability
/// (CR 606.3).
pub const LOYALTY_ABILITY: &str = "loyalty ability";

/// `Filter::Custom` name: an activated ability on the stack with {X} in its activation
/// cost.
pub const X_IN_ACTIVATION_COST: &str = "ability with X in its activation cost";

/// The activated ability the stack object `id` is, if it is one.
fn activated(g: &Game, id: ObjectId) -> Option<&ActivatedAbility> {
    match g.obj(id).stack.as_deref().map(|si| &si.kind) {
        Some(StackKind::Activated { ability, .. }) => match &ability.kind {
            AbilityKind::Activated(a) => Some(a),
            _ => None,
        },
        _ => None,
    }
}

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
    match name {
        LOYALTY_ABILITY => Some(activated(g, id).is_some_and(|a| a.is_loyalty)),
        X_IN_ACTIVATION_COST => {
            Some(activated(g, id).is_some_and(|a| a.cost.mana.as_ref().is_some_and(|m| m.has_x())))
        }
        _ => None,
    }
}
