//! "[Permanents] can't be turned face up" (Karlov Watchdog: "Permanents your opponents
//! control can't be turned face up during your turn"; Unable to Scream: "As long as
//! enchanted creature is face down, it can't be turned face up"): a face-down permanent
//! such an effect applies to can't be turned face up in any way (CR 708.7): not by paying
//! its morph or disguise cost (CR 702.37e, 702.168d), not by paying the mana cost of a
//! manifested or cloaked creature card (CR 701.40b, 701.58b), and not by an effect. Its
//! controller can't even attempt it (Karlov Watchdog ruling), so no cost is paid.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::ObjectId;

/// Whether an effect says the face-down permanent `id` can't be turned face up.
pub fn cant_be_turned_face_up(g: &Game, id: ObjectId) -> bool {
    g.statics.restrictions.iter().any(|(s, c, r)| match r {
        Restriction::CantTurnFaceUp(f) => g.matches(id, f, &Ctx::new(Some(*s), *c)),
        _ => false,
    }) || g.rule_effects.iter().any(|e| match &e.restriction {
        Restriction::CantTurnFaceUp(f) => {
            e.objects.as_ref().is_none_or(|v| v.contains(&id))
                && g.matches(id, f, &Ctx::new(e.source, e.controller))
        }
        _ => false,
    })
}
