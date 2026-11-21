//! "This spell can't be copied." (See Double, Display of Power, Choreographed Sparks) and
//! "This ability can't be copied." (Gogo, Master of Mimicry): such a spell or ability
//! can't be copied (CR 707.10); the instruction functions while it's on the stack
//! (CR 113.6g), including as it last existed there when an effect copies it after it
//! left (CR 608.2h). An effect that would copy it creates no copy.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::{ObjKind, StackKind};
use crate::types::ObjectId;

/// Whether the spell or ability on the stack `id` (or as it last existed there) can't be
/// copied.
pub fn cant_be_copied(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    // "This ability can't be copied."
    if let Some(StackKind::Activated { ability, .. }) = o.stack.as_deref().map(|s| &s.kind) {
        if matches!(&ability.kind, AbilityKind::Activated(a) if a.cant_be_copied) {
            return true;
        }
    }
    // The spell's own "This spell can't be copied." (also as it last existed on the
    // stack, when its static abilities are no longer collected).
    let own = o.kind != ObjKind::StackAbility
        && o.chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if s.zone == FunctionZone::Stack
                && matches!(&s.effect, StaticEffect::Restriction(Restriction::CantBeCopied(Filter::Source))))
    });
    own || g.statics.restrictions.iter().any(|(s, c, r)| match r {
        Restriction::CantBeCopied(f) => g.matches(id, f, &Ctx::new(Some(*s), *c)),
        _ => false,
    })
}
