//! Shared helpers for the tests of CR 702.140–702.152 (mutate, encore, boast, foretell,
//! demonstrate, daybound, disturb, decayed, cleave, training, compleated, reconfigure,
//! blitz).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, StackKind};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// Adds mana of one type to a player's mana pool.
pub fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Total mana in a player's mana pool.
pub fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// Whether casting `card` for `p` with `method` is currently a legal action.
pub fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).iter().any(
        |a| matches!(a, Action::Cast { card: c, method: m } if *c == card && *m == method),
    )
}

/// Whether activating the ability `uid` of `source` is currently a legal action for `p`.
pub fn activatable(t: &mut TestGame, p: PlayerId, source: ObjectId, uid: u64) -> bool {
    t.g.turn.priority = Some(p);
    let source = t.g.current(source);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Activate { source: s, ability } if *s == source && *ability == uid)
    })
}

/// The uid of the first activated ability of `source` whose text starts with `prefix`.
pub fn ability_uid(t: &mut TestGame, source: ObjectId, prefix: &str) -> u64 {
    t.g.recompute();
    let source = t.g.current(source);
    t.g.obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no activated ability starting with {prefix:?}"))
}

/// Activates the ability `uid` of `source` for `p` (with priority).
pub fn activate_uid(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    uid: u64,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let source = t.g.current(source);
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, source, uid);
    t.g.flush_events();
    r
}

/// Declares attackers for the active player (from the beginning of combat) and stops once
/// attack triggers are on the stack, in the declare attackers step.
pub fn declare_attackers(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
}

/// Advances the current turn to `step` (the active player has priority).
pub fn to_step(t: &mut TestGame, step: Step) {
    let ap = t.g.turn.active;
    t.advance_to(ap, step);
}

/// Triggered abilities on the stack whose text starts with `prefix`.
pub fn triggers_on_stack(t: &TestGame, prefix: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            matches!(&t.g.obj(**id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { ability, .. }) if ability.text.starts_with(prefix))
        })
        .count()
}

/// Whether the object (followed across zone changes) has the keyword.
pub fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

/// Creature tokens a player controls on the battlefield.
pub fn creature_tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = t.g.obj(*id);
            o.controller == p && o.is_token() && o.is_creature()
        })
        .collect()
}

/// Whether the creature could be declared as an attacker now.
pub fn can_attack_now(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == id)
}

/// Whether the creature could be declared as a blocker of `attacker` now (after attackers
/// are declared).
pub fn can_block_now(t: &mut TestGame, blocker: ObjectId, attacker: ObjectId) -> bool {
    t.g.recompute();
    let (b, a) = (t.g.current(blocker), t.g.current(attacker));
    let dp = t.g.obj(b).controller;
    mtg_engine::combat::block_options(&t.g, &[dp])
        .iter()
        .any(|(c, attackers)| *c == b && attackers.contains(&a))
}
