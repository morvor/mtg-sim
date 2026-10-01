//! Shared helpers for the tests of rulings batch P076 (`r_p076_*.rs`): cards that give
//! trample, "can't be blocked", "can't be countered", and vigilance.

#![allow(dead_code)]

use crate::r_s21_common::blocks_now;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Whether `attacker` is blocked in the current combat (CR 509.1h).
pub fn is_blocked(t: &TestGame, attacker: ObjectId) -> bool {
    let a = t.g.current(attacker);
    t.g.combat
        .as_ref()
        .and_then(|c| c.attackers.iter().find(|x| x.id == a))
        .is_some_and(|x| x.blocked)
}

/// P0 attacks P1 with `attacker`, P1 blocks it with `blocker`, then `act` runs in the
/// declare blockers step (an ability that makes the attacker unblockable). Its effects
/// resolve, and the attacker must still be blocked and deal no damage to P1 (CR 509.1h:
/// a blocked creature stays blocked).
pub fn unblockable_after_blocked(
    t: &mut TestGame,
    attacker: ObjectId,
    blocker: ObjectId,
    act: impl FnOnce(&mut TestGame),
) {
    crate::r_s03_common::to_blockers(t, &[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    assert!(is_blocked(t, attacker));
    act(t);
    t.resolve_all();
    assert!(is_blocked(t, attacker), "the attacker became unblocked");
    assert!(blocks_now(t).contains(&(blocker, t.g.current(attacker))));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "the blocked attacker dealt damage to the player");
}

/// Hand size of `p`.
pub fn hand_count(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).hand.len()
}
