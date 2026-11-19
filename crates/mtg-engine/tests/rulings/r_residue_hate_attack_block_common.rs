//! Shared helpers for the tests of the rulings residue item
//! `rulings-residue-hate-attack-block` (`r_residue_hate_attack_block_*.rs`): the open
//! rulings left in batches P079, P080 and P082 (cards that punish attackers and blockers,
//! attack and block requirements and restrictions). The helpers of the S and P batches are
//! used too.

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// An entity for an object.
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0 attacks P1 with `attackers` and P1 declares `blocks` (blocker, attacker); stops in
/// the declare blockers step with P0 holding priority, block triggers on the stack.
pub fn blocks(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let at: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    crate::r_s03_common::to_blockers(t, &at, blocks);
}

/// P0 attacks with `attackers` (each at its entity) and the game stops in the declare
/// attackers step with P0 holding priority.
pub fn attacks(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    crate::r_s01_common::attack_with(t, attackers);
}

/// The attackers `blocker` (followed across zones) is blocking now.
pub fn blocking(t: &TestGame, blocker: ObjectId) -> Vec<ObjectId> {
    let b = t.g.current(blocker);
    t.g.combat
        .as_ref()
        .map(|c| c.blocking(b))
        .unwrap_or_default()
}

/// Whether the object (followed across zones) is attacking now.
pub fn attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_attacking(t.g.current(id))
}

/// Whether the object (followed across zones) is tapped.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// The controller of the object (followed across zones) now.
pub fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Whether the object (followed across zones) is now in `zone`.
pub fn in_zone(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    t.zone(id) == zone
}

/// Taps the object (as an effect would: the primitive, with its events).
pub fn tap(t: &mut TestGame, id: ObjectId) {
    crate::r_p209_common::tap(t, id);
}

/// Untaps the object (as an effect would: the primitive, with its events).
pub fn untap(t: &mut TestGame, id: ObjectId) {
    crate::r_p209_common::untap(t, id);
}

/// From wherever the game is, plays on (everyone passing) to `p`'s next untap step and
/// through it, stopping in that turn's upkeep.
pub fn through_untap_of(t: &mut TestGame, p: PlayerId) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(20_000, |g| {
        g.turn.number != turn && g.turn.active == p && g.turn.step == Step::Upkeep
    });
    assert!(ok, "did not reach {p}'s upkeep");
}

/// Declares no blockers for `p` (queued).
pub fn no_blocks(t: &mut TestGame, p: PlayerId) {
    t.answer(p, DecisionKind::Blockers, Answer::Blockers(vec![]));
}

/// Gives control of the object to `p` (as a control-changing effect with no end would).
pub fn give_control(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    crate::r_s06_common::give_control(t, t.g.current(id), p);
}
