//! Shared helpers for the tests of rulings batch P130 (`r_p130_*.rs`): reflexive triggered
//! abilities ("When you do, ...", CR 603.12) and mana abilities / abilities that add mana
//! (CR 605). (The helpers of batches S01–S30 and P108 are used too.)

#![allow(dead_code)]

use mtg_engine::object::StackKind;
use mtg_engine::types::counters;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{obj, resolved};
pub use crate::r_s01_common::{asked_since, attack_with, block_and_finish, supported, watch};
pub use crate::r_s02_common::{
    can_activate, can_cast, can_play_land, create_token, destroy, target_candidates,
};
pub use crate::r_s25_common::{cast_new, lands_for_cost, targets_of};

/// The top object of the stack.
pub fn top(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("the stack is empty")
}

/// Whether the object on the stack is a triggered ability.
pub fn is_trigger(t: &TestGame, id: ObjectId) -> bool {
    matches!(
        t.g.obj(id).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    )
}

/// Asserts that the top of the stack is a triggered ability (or the spell or ability
/// `what` describes) with no targets, and that `p` wasn't asked to choose any target since
/// decision `from`.
pub fn no_target_yet(t: &TestGame, p: PlayerId, from: usize) {
    assert!(t.stack_len() >= 1, "nothing on the stack");
    assert!(targets_of(t, top(t)).is_empty(), "a target was chosen");
    assert!(
        target_candidates(t, p, from).is_empty(),
        "a target choice was asked"
    );
}

/// Resolves the top of the stack (the ability or spell that may pay / sacrifice / discard
/// ...) and asserts that a reflexive triggered ability went on the stack above what was
/// there before, with exactly the targets `expect`; returns it. The effect hasn't happened
/// yet: players get priority with it on the stack (CR 603.12, 117.3b).
pub fn resolve_into_reflexive(t: &mut TestGame, expect: &[Entity]) -> ObjectId {
    let before = t.stack_len();
    t.resolve();
    assert_eq!(t.stack_len(), before, "no reflexive trigger went on the stack");
    let r = top(t);
    assert!(is_trigger(t, r), "the top of the stack isn't a trigger");
    assert_eq!(targets_of(t, r), expect.to_vec());
    r
}

/// Mana of each kind in `p`'s pool, in W, U, B, R, G, C order.
pub fn pool(t: &TestGame, p: PlayerId) -> [usize; 6] {
    use mtg_engine::mana::ManaType;
    ManaType::ALL.map(|ty| t.g.player(p).mana_pool.count(ty))
}

/// Total mana in `p`'s pool.
pub fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// From the declare attackers step (no blocks), advances into the combat damage step and
/// puts the combat damage triggers on the stack.
pub fn to_combat_damage_triggers(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, mtg_engine::turn::Step::CombatDamage);
    t.settle();
}

/// Sets the loyalty of a planeswalker.
pub fn loyalty(t: &mut TestGame, pw: ObjectId, n: u32) {
    let pw = t.g.current(pw);
    let have = t.counters(pw, counters::LOYALTY);
    if n > have {
        t.g.add_counters(Entity::Object(pw), counters::LOYALTY, n - have, None);
    } else {
        t.g.remove_counters(Entity::Object(pw), counters::LOYALTY, have - n);
    }
    t.g.recompute();
}
