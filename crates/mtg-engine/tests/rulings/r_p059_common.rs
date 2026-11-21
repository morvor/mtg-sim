//! Shared helpers for the tests of rulings batch P059 (`r_p059_*.rs`): flicker effects that
//! return the card later (at the beginning of the next end step), fling effects, flowstone
//! pumps, and "attacks (or blocks) this turn if able" requirements. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use crate::r_s01_common::custom_card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// A historic, nonland permanent (a legendary artifact creature) with "At the beginning
/// of each end step, you gain 1 life." under `p`'s control.
pub fn end_step_keeper(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = custom_card(
        "Dusk Keeper",
        "Legendary Artifact Creature — Construct",
        "{2}",
        Some((1, 1)),
        "At the beginning of each end step, you gain 1 life.",
    );
    t.custom(p, def, Zone::Battlefield)
}

/// Whether the token is gone: neither on the battlefield nor in exile (CR 111.8).
pub fn token_gone(t: &TestGame, token: ObjectId) -> bool {
    let z = t.zone(token);
    z != Zone::Battlefield && !t.g.exile.contains(&t.g.current(token))
}

/// Advances to the beginning of `p`'s end step and resolves everything there.
pub fn end_step_of(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.resolve_all();
}

/// Runs the game to `step` of the current turn, with priority, and settles.
pub fn run_to(t: &mut TestGame, step: Step) {
    use mtg_engine::turn::Stage;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority) || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?}");
    t.settle();
}

/// From the declare attackers step: `dp` declares these blocks; the declare blockers step
/// begins (its triggers on the stack).
pub fn block(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    use mtg_engine::decision::Answer;
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    run_to(t, Step::DeclareBlockers);
}

/// Puts `n` counters of `kind` on `id`.
pub fn put(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}

/// `p` casts the real spell `name` (with the mana for it) with the given targets (one per
/// slot), leaving it on the stack.
pub fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    crate::r_s01_common::supported(name);
    crate::r_s01_common::give_mana_for(t, p, name);
    let s = t.hand(p, name);
    let mut b = t.cast(p, s);
    for e in targets {
        b = b.target(*e);
    }
    b.go()
}

/// `p` casts the real spell `name` and everything resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let s = cast(t, p, name, targets);
    t.resolve_all();
    s
}

/// The legal choices for target slot `slot` of the `index`th activated ability of the
/// permanent `source` (controlled by its controller).
pub fn ability_target_choices(
    t: &mut TestGame,
    source: ObjectId,
    index: usize,
    slot: usize,
) -> Vec<Entity> {
    use mtg_engine::ability::AbilityKind;
    t.g.recompute();
    let s = t.g.current(source);
    let o = t.g.obj(s);
    let spec = o
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(x) => Some(x),
            _ => None,
        })
        .nth(index)
        .and_then(|a| a.body.targets.get(slot).cloned())
        .expect("no such target");
    let ctx = mtg_engine::eval::Ctx::new(Some(s), o.controller);
    t.g.legal_target_candidates(&spec, &ctx, s)
}
