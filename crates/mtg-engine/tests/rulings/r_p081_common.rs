//! Shared helpers for the tests of rulings batch P081 (`r_p081_*.rs`): cards that punish
//! attackers and blockers ("hate-attacker" cards): "blocks or becomes blocked" triggers,
//! attack triggers, damage divided among attackers, protection from colors, and the
//! spells that care about attacking or blocking creatures.

#![allow(dead_code)]
#![allow(unused_imports)]

pub use crate::r_p006_common::{mark, obj, old, pl, priority_since, top};
pub use crate::r_p082_common::{block_with, blocked, finish_combat, is_blocked, triggers_from};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, give_mana_for, supported, tokens, watch,
};
pub use crate::r_s04_common::{ability_targets, spell_targets};
pub use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// P1 attacks P0 with each of `attackers`; P0 declares `blocks` (blocker, attacker). Stops
/// in the declare blockers step with P1 holding priority, block triggers on the stack.
pub fn blocked_by_p0(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    t.set_step(P1, Step::BeginningOfCombat);
    let at: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, pl(P0))).collect();
    crate::r_s03_common::to_blockers(t, &at, blocks);
}

/// P1 attacks P0 with each of `attackers` (no blocks yet): stops in the declare attackers
/// step of P1's turn with the attack triggers on the stack.
pub fn attacked_by_p1(t: &mut TestGame, attackers: &[ObjectId]) {
    t.set_step(P1, Step::BeginningOfCombat);
    let at: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, pl(P0))).collect();
    attack_with(t, &at);
}

/// Whether the permanent (followed across zone changes) is tapped now.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Whether the permanent (followed across zone changes) is phased out now.
pub fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// The damage marked on the permanent (followed across zone changes) now.
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// Whether the object is in its owner's graveyard now (followed across zone changes).
pub fn died(t: &TestGame, id: ObjectId) -> bool {
    matches!(t.zone(id), Zone::Graveyard(_))
}

/// The number of counters of `kind` a player has (energy, poison, ...).
pub fn player_counters(t: &TestGame, p: PlayerId, kind: &str) -> u32 {
    t.g.player(p)
        .counters
        .iter()
        .find(|(k, _)| k.as_str() == kind)
        .map(|(_, n)| *n)
        .unwrap_or(0)
}

/// Whether the spell `name`, put into `p`'s hand with mana for it, could be cast by `p`
/// now (the card stays in hand).
pub fn can_cast_now(t: &mut TestGame, p: PlayerId, name: &str) -> bool {
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    crate::r_s02_common::can_cast(t, p, card, mtg_engine::object::CastMethod::Normal)
}

/// Whether `p` could choose `target` for the first target of the spell `name` if `p` cast
/// it now (the card is put into `p`'s hand).
pub fn can_target_with(t: &mut TestGame, p: PlayerId, name: &str, target: Entity) -> bool {
    spell_targets(t, p, name).contains(&target)
}

/// The colors of the object (followed across zone changes) now.
pub fn colors(t: &TestGame, id: ObjectId) -> Vec<Color> {
    let o = t.obj_now(id);
    Color::ALL
        .iter()
        .copied()
        .filter(|c| o.chars.colors.contains(*c))
        .collect()
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has_keyword(t: &TestGame, id: ObjectId, k: mtg_engine::keywords::KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

/// Transforms a double-faced permanent (as an effect would).
pub fn transform(t: &mut TestGame, id: ObjectId) {
    crate::r_s17_common::transform(t, id)
}

/// Whether `p` was asked for priority in `step` since decision `from` (the step is read
/// from the game when the decision is asked).
pub fn priority_steps(t: &mut TestGame, p: PlayerId) -> crate::r_s01_common::Seen<Step> {
    watch(
        t,
        p,
        |d| matches!(d, Decision::Priority { .. }),
        |g| g.turn.step,
    )
}
