//! Shared helpers for the tests of rulings batch P122 (`r_p122_*.rs`): spells and
//! abilities that stop creatures from attacking or blocking. (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use crate::r_p076_common::is_blocked;
use crate::r_s01_common::attack_with;
use crate::r_s03_common::to_blockers;
use crate::r_s09_common::to_combat;
use crate::r_s10_common::attacking;
use crate::r_s21_common::{blocks_now, legal_blocks};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0 attacks P1 with `attacker` (P1 doesn't block); then, in the declare attackers step,
/// `act` runs and everything resolves. The attacker must still be attacking and deal its
/// combat damage to P1 (CR 506.4, 508.1: restrictions on attacking are checked only as
/// attackers are declared).
pub fn still_attacks_after(t: &mut TestGame, attacker: ObjectId, act: impl FnOnce(&mut TestGame)) {
    to_combat(t, P0);
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    assert!(attacking(t, attacker));
    act(t);
    t.resolve_all();
    assert!(attacking(t, attacker), "the attacker was removed from combat");
    // The effect did apply: the creature couldn't attack now even if it were untapped.
    let a = t.g.current(attacker);
    let tapped = std::mem::replace(&mut t.g.objects[a.0 as usize].tapped, false);
    let could = t.g.can_attack(a);
    t.g.objects[a.0 as usize].tapped = tapped;
    assert!(!could, "the effect didn't apply");
    let power = t.pt(attacker).0;
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - power, "the attacker didn't deal combat damage");
}

/// P0 attacks P1 with `attacker` and P1 blocks it with `blocker`; then, in the declare
/// blockers step, `act` runs and everything resolves. The blocker must still be blocking
/// the attacker, which stays blocked and deals no damage to P1, and the blocker deals its
/// combat damage to the attacker (CR 506.4, 509.1: restrictions on blocking are checked
/// only as blockers are declared; CR 509.1h).
pub fn still_blocks_after(
    t: &mut TestGame,
    attacker: ObjectId,
    blocker: ObjectId,
    act: impl FnOnce(&mut TestGame),
) {
    to_combat(t, P0);
    to_blockers(t, &[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    assert!(blocks_now(t).contains(&(blocker, attacker)));
    act(t);
    t.resolve_all();
    let (a, b) = (t.g.current(attacker), t.g.current(blocker));
    assert!(is_blocked(t, a), "the attacker became unblocked");
    let blocker_alive = t.on_battlefield(b);
    if blocker_alive {
        assert!(blocks_now(t).contains(&(b, a)), "the block was undone");
        // The effect did apply: this block couldn't be declared now.
        assert!(!legal_blocks(t, P1, &[(b, a)]), "the effect didn't apply");
    }
    let blocker_power = if blocker_alive { t.pt(b).0 } else { 0 };
    let before = t.g.obj(a).damage;
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "the blocked attacker dealt damage to the player");
    if blocker_alive && t.on_battlefield(a) {
        assert_eq!(
            t.g.obj(a).damage,
            before + blocker_power as u32,
            "the blocker didn't deal combat damage"
        );
    }
}
