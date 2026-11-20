//! Liveness of object ids in the primitives (CR 400.7): "An object that moves from one
//! zone to another becomes a new object with no memory of, or relation to, its previous
//! existence." Every zone change makes a new `ObjectId`; the old id keeps the object's last
//! known information.
//!
//! The primitives of `actions.rs` that act on an object (tap, untap, damage, counters,
//! attach, move, destroy, sacrifice) do nothing to an object that has left its zone, which
//! is what the rules have an instruction do to it. But an id from before a zone change
//! reaching a primitive is more often a mistake than a deliberate reference to a departed
//! object: code that should have followed the object to its new zone (CR 400.7j: "other
//! parts of that effect can find that object") or that should have read last known
//! information instead. So in debug builds the primitives assert that the ids they're given
//! are live ([`Game::assert_live`]), and code that knowingly holds ids of objects that may
//! have left says so by passing them through [`Game::present`] / [`Game::all_present`]
//! (an instruction about an object that has left does nothing to it, CR 400.7, 608.2b).

use crate::game::Game;
use crate::types::{Entity, ObjectId};

impl Game {
    /// Debug check that a primitive acting on `id` (`action`) is given the object as it is
    /// now, not one that has left its zone (see the module docs).
    #[track_caller]
    #[inline]
    pub(crate) fn assert_live(&self, id: ObjectId, action: &str) {
        debug_assert!(
            self.is_live(id),
            "{action} on {id:?} ({:?}, {:?}), which has left its zone (now {:?} in {:?}); \
             follow it with `current`, or pass ids that may have left through `present` \
             (CR 400.7)",
            self.obj(id).chars.name,
            self.obj(id).zone,
            self.current(id),
            self.obj(self.current(id)).zone,
        );
    }

    /// `id` if it's still the object it was (it hasn't left its zone), for an instruction
    /// about an object that may have left: such an instruction does nothing to it, nor to
    /// the new object it became (CR 400.7).
    pub fn present(&self, id: ObjectId) -> Option<ObjectId> {
        self.is_live(id).then_some(id)
    }

    /// The objects of `ids` that haven't left their zones ([`Game::present`]).
    pub fn all_present(&self, ids: impl IntoIterator<Item = ObjectId>) -> Vec<ObjectId> {
        ids.into_iter().filter(|o| self.is_live(*o)).collect()
    }

    /// `e` unless it's an object that has left its zone ([`Game::present`]); players stay.
    pub fn present_entity(&self, e: Entity) -> Option<Entity> {
        match e {
            Entity::Object(o) if !self.is_live(o) => None,
            other => Some(other),
        }
    }

    /// The entities of `es` that aren't objects that have left their zones.
    pub fn all_present_entities(&self, es: impl IntoIterator<Item = Entity>) -> Vec<Entity> {
        es.into_iter()
            .filter_map(|e| self.present_entity(e))
            .collect()
    }
}
