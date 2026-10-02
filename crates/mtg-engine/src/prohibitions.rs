//! Prohibitions on what can happen to objects ([`Restriction::CantBe`]): "can't become
//! untapped" (no untapping at all, CR 701.26b), "can't phase in" (CR 702.26), "can't be
//! turned face up" (CR 708.8), "can't be equipped" (CR 301.5c), "can't be enchanted by
//! other Auras" (CR 303.4), "can't become suspected" (CR 701.60). Each check is a hook
//! where the engine would otherwise perform that action.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::ObjectId;

/// The restrictions of this kind that apply to `id`: (source, controller).
fn applying(g: &Game, id: ObjectId, action: ObjectAction) -> Vec<Option<ObjectId>> {
    g.all_restrictions()
        .into_iter()
        .filter(|(s, c, r, locked)| match r {
            Restriction::CantBe { what, action: a } if *a == action => {
                g.restriction_applies(id, what, &Ctx::new(*s, *c), locked)
            }
            _ => false,
        })
        .map(|(s, ..)| s)
        .collect()
}

/// Whether `action` can't happen to `id`.
pub fn object_cant(g: &Game, id: ObjectId, action: ObjectAction) -> bool {
    if !g.is_live(id) {
        return false;
    }
    !applying(g, id, action).is_empty()
}

/// Whether the Aura `aura` can't enchant `target` because of "can't be enchanted by
/// other Auras" (an Aura with that ability can still enchant it).
pub fn aura_prohibited(g: &Game, aura: ObjectId, target: ObjectId) -> bool {
    if !g.is_live(target) {
        return false;
    }
    applying(g, target, ObjectAction::EnchantedByOtherAuras)
        .iter()
        .any(|s| *s != Some(aura))
}
