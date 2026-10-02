//! "Counters remain on ~ as it moves to any zone other than a player's hand or library."
//! (Skullbriar, the Walking Grave; Me, the Immortal): an exception to CR 122.2 ("Counters
//! on an object are not retained if that object moves from one zone to another") and to
//! CR 400.7. The ability functions in every zone.
//!
//! * The new object has the counters the old one had as it moved. They aren't "put" on
//!   it: no replacement effect modifies them and nothing triggers on them (Skullbriar and
//!   Me rulings: "Effects like Doubling Season's ... won't affect those counters").
//! * The ability works only if the object has it in the zone it's moving from (a
//!   Skullbriar in a graveyard under Yixlid Jailer loses its counters as it leaves).
//! * Counters that adjust power and toughness apply in every zone (CR 122.1a, see
//!   `layers.rs`), so a Skullbriar in the command zone with a +1/+1 counter is 2/2.

use crate::ability::*;
use crate::game::Game;
use crate::object::Zone;
use crate::types::ObjectId;

/// Whether the object `id` has a functioning "counters remain" ability.
fn has_ability(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.chars.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Static(s) => {
            matches!(s.effect, StaticEffect::CountersRemain)
                && g.ability_functions(o, s.zone, s.is_cda)
        }
        _ => false,
    })
}

/// Called as the object `old` moves to `to`, becoming `new` (CR 400.7): its counters
/// remain on the new object if it has the ability and isn't going to a hand or library.
pub fn follow(g: &mut Game, old: ObjectId, new: ObjectId, to: Zone) {
    if matches!(to, Zone::Hand(_) | Zone::Library(_) | Zone::Nowhere) {
        return;
    }
    let o = g.obj(old);
    if o.counters.is_empty() || !has_ability(g, old) {
        return;
    }
    let counters = o.counters.clone();
    let stamps = o.counter_timestamps.clone();
    let n = &mut g.objects[new.0 as usize];
    n.counters = counters;
    n.counter_timestamps = stamps;
    g.dirty = true;
}
