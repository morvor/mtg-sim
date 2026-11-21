//! Shared helpers for the tests of rulings batch P058 (`r_p058_*.rs`): firebreathing-style
//! pump abilities and flicker effects ("exile it, then return it to the battlefield").
//! (The helpers of batches S01–S32 are used too.)

#![allow(dead_code)]

use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Basic lands for `p`: `n` of each basic land type plus `n` Wastes.
pub fn mana(t: &mut TestGame, p: PlayerId, n: usize) {
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"] {
        t.lands(p, l, n);
    }
}

/// What was on a permanent before it was flickered.
pub struct Decorated {
    pub id: ObjectId,
    pub aura: ObjectId,
    pub equipment: ObjectId,
}

/// Puts a +1/+1 counter on the permanent `id`, and attaches a Holy Strength (Aura) and a
/// Bonesplitter (Equipment) controlled by `p` to it.
pub fn decorate(t: &mut TestGame, p: PlayerId, id: ObjectId) -> Decorated {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, 1, None);
    let aura = attach_new(t, p, "Holy Strength", id);
    let equipment = attach_new(t, p, "Bonesplitter", id);
    t.g.recompute();
    t.settle();
    assert_eq!(t.counters(id, counters::PLUS1), 1);
    Decorated {
        id,
        aura,
        equipment,
    }
}

/// The decorated permanent was exiled and returned: it's a new object on the battlefield,
/// without the counter; the Aura went to its owner's graveyard; the Equipment stayed on
/// the battlefield unattached (CR 400.7, 704.5m, 704.5n).
pub fn assert_new_object(t: &TestGame, d: &Decorated) {
    let now = t.g.current(d.id);
    assert_ne!(now, d.id, "a new object");
    assert!(t.on_battlefield(now), "returned: {}", t.dump_log());
    assert_eq!(t.counters(now, counters::PLUS1), 0, "counters cease to exist");
    let aura_owner = t.obj_now(d.aura).owner;
    assert_eq!(t.zone(d.aura), Zone::Graveyard(aura_owner), "Aura");
    assert!(t.on_battlefield(d.equipment), "Equipment stays");
    assert_eq!(attached_to(t, d.equipment), None, "Equipment unattached");
}

/// Advances from `p`'s main phase to `p`'s end step and resolves the triggers there.
pub fn through_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, mtg_engine::turn::Step::End);
    t.resolve_all();
}
