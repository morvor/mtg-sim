//! Shared helpers for rulings batch P036 (`r_p036_*.rs`): counterspells (of spells and of
//! abilities) and a few counters-matter cards.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P1 casts Abrupt Decay ("This spell can't be countered. Destroy target nonland permanent
/// with mana value 3 or less.") at P0's Grizzly Bears. Returns (the spell, the Bears).
pub fn uncounterable_decay(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let decay = crate::r_s25_common::cast_new(t, P1, "Abrupt Decay", &[obj(bears)]);
    (decay, bears)
}

/// P1 casts Carnage Tyrant ("This spell can't be countered. Trample, hexproof", a 7/6).
pub fn uncounterable_tyrant(t: &mut TestGame) -> ObjectId {
    turn_of(t, P1);
    crate::r_s25_common::cast_new(t, P1, "Carnage Tyrant", &[])
}

/// Makes it `p`'s precombat main phase (so `p` can cast creature spells), unless it
/// already is `p`'s turn.
pub fn turn_of(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active != p {
        t.set_step(p, mtg_engine::turn::Step::PrecombatMain);
    }
}

/// `p` casts Grizzly Bears (a creature spell). Returns the spell.
pub fn bears_spell(t: &mut TestGame, p: PlayerId) -> ObjectId {
    turn_of(t, p);
    crate::r_s25_common::cast_new(t, p, "Grizzly Bears", &[])
}

/// The candidate lists of every target choice asked of `p` since decision `from`.
pub fn offered(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Whether the object `id` is on the stack now.
pub fn on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.g.stack.contains(&id)
}
