//! Descriptions of activated abilities on the stack, for triggers qualified by the kind of
//! ability activated: "Whenever you activate a loyalty ability of a Chandra planeswalker"
//! (Chandra's Regulator, CR 606), "When you next ... activate an ability with {X} in its
//! activation cost this turn" (Magus Lucea Kane, CR 107.3), "causes a triggered ability of
//! that creature to trigger" (Aboleth Spawn, CR 603.3b).

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

/// `Filter::Custom` name: a triggered ability on the stack whose source is the object its
/// trigger event is about ("a creature entering causes a triggered ability of that
/// creature to trigger").
pub const OF_THE_TRIGGERING_OBJECT: &str = "triggered ability of the object its event is about";

/// Whether the triggered ability `id` on the stack is an ability of the object its trigger
/// event is about.
fn of_the_triggering_object(g: &Game, id: ObjectId) -> bool {
    let Some(si) = g.obj(id).stack.as_deref() else {
        return false;
    };
    let StackKind::Triggered { source, .. } = &si.kind else {
        return false;
    };
    let Some(info) = &si.event else {
        return false;
    };
    let source = g.current(*source);
    [info.object, info.lki]
        .into_iter()
        .flatten()
        .any(|o| g.current(o) == source)
}

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
        OF_THE_TRIGGERING_OBJECT => Some(of_the_triggering_object(g, id)),
        _ => None,
    }
}
