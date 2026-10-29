//! Shared helpers for the tests of rulings batch P063 (`r_p063_*.rs`): tapping effects
//! with "doesn't untap" and stun counters, cost increases and reductions, mana refunds,
//! "play with the top card of your library revealed", and creatures that gain first
//! strike, double strike or flying. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The object as an entity (a target or a choice).
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn pool(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has(t: &mut TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(kw)
}

/// Puts `n` counters of `kind` on the object and settles.
pub fn put(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    crate::r_s29_common::put_counters(t, id, kind, n);
}

/// P0 attacks P1 with `attackers`, P1 declares `blocks`, and the game advances to the
/// combat damage step with first-strike damage dealt (if any creature had first or double
/// strike) and the triggers there resolved. P0 has priority in the first-strike damage
/// step (or in the regular one, if there was no first-strike step).
pub fn to_first_strike_damage(
    t: &mut TestGame,
    attackers: &[ObjectId],
    blocks: &[(ObjectId, ObjectId)],
) {
    let at: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    crate::r_s03_common::to_blockers(t, &at, blocks);
    t.resolve_all();
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        matches!(g.turn.step, Step::FirstStrikeDamage | Step::CombatDamage)
            && g.turn.stage == mtg_engine::turn::Stage::Priority
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "no combat damage step");
    t.resolve_all();
}

/// Finishes P0's combat (advances to the end of combat step).
pub fn finish_combat(t: &mut TestGame) {
    t.advance_to(P0, Step::EndOfCombat);
}
