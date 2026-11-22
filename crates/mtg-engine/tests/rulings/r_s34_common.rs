//! Shared helpers for the tests of rulings batch S34 (`r_s34_*.rs`): mana value (CR
//! 202.3) — what it's determined from (the mana cost only, never the cost paid), {X} on
//! and off the stack (CR 202.3e, 107.3g), Phyrexian and split mana costs, and land
//! cards (CR 202.3a). (The helpers of batches S01–S29 are used too.)

#![allow(dead_code)]

use crate::r_s08_common::mana_value;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The alternative-cost method P0 could cast `card` with now.
pub fn alternative(t: &mut TestGame, card: ObjectId) -> CastMethod {
    crate::r_s07_common::cast_methods(t, P0, card)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("an alternative cost")
}

/// Endless One ({X} 0/0, "This creature enters with X +1/+1 counters on it."), cast by
/// `p` with X = `x` and resolved: an X/X whose mana value on the battlefield is 0.
pub fn endless_one(t: &mut TestGame, p: PlayerId, x: usize) -> ObjectId {
    t.lands(p, "Wastes", x);
    let card = t.hand(p, "Endless One");
    t.cast(p, card).x(x as i64).go();
    t.resolve_all();
    let id = t.g.current(card);
    assert!(t.on_battlefield(id));
    assert_eq!(t.pt(id), (x as i32, x as i32));
    assert_eq!(mana_value(t, id), 0);
    id
}

/// P1's Endless One cast with X = `x` (in P1's main phase); then it's P0's main phase.
pub fn p1_endless_one(t: &mut TestGame, x: usize) -> ObjectId {
    use mtg_engine::turn::Step;
    t.set_step(P1, Step::PrecombatMain);
    let id = endless_one(t, P1, x);
    t.set_step(P0, Step::PrecombatMain);
    id
}

/// Kaervek the Merciless under P1's control: "Whenever an opponent casts a spell, Kaervek
/// deals damage equal to that spell's mana value to any target." Call
/// [`aim_kaervek_at_p0`] before each spell P0 casts.
pub fn kaervek(t: &mut TestGame) -> ObjectId {
    crate::r_s01_common::supported("Kaervek the Merciless");
    t.battlefield(P1, "Kaervek the Merciless")
}

/// Queues P1's target for the next Kaervek trigger: P0.
pub fn aim_kaervek_at_p0(t: &mut TestGame) {
    t.answer_targets(P1, &[Entity::Player(P0)]);
}

/// The full party: a Cleric, a Rogue, a Warrior, and a Wizard under `p`'s control
/// (CR 700.8).
pub fn full_party(t: &mut TestGame, p: PlayerId) {
    for name in [
        "Kinjalli's Caller",
        "Snooping Newsie",
        "Elvish Warrior",
        "Prodigal Sorcerer",
    ] {
        t.battlefield(p, name);
    }
}
