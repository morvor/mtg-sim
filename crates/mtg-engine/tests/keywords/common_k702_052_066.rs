//! Shared helpers for the tests of CR 702.52–702.66 (dredge, transmute, bloodthirst,
//! haunt, replicate, forecast, graft, recover, ripple, split second, suspend, vanishing,
//! absorb, aura swap, delve).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts a custom card onto the battlefield through a real zone change (replacement
/// effects such as "enters with counters" apply, and enters abilities trigger).
pub fn enter_def(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    let id = t.custom(p, def, Zone::Nowhere);
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(p),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    })
    .expect("failed to enter the battlefield")
}

/// Executes an effect as if an ability of `source` controlled by `controller` resolved,
/// with `targets` in target slot 0.
pub fn run_effect(
    t: &mut TestGame,
    source: Option<ObjectId>,
    controller: PlayerId,
    effect: Effect,
    targets: &[Entity],
) {
    let mut ctx = mtg_engine::eval::Ctx::new(source, controller);
    ctx.targets = vec![targets.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// Removes `n` counters of `kind` from `id` (as an effect would).
pub fn remove_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: i32) {
    let id = t.g.current(id);
    run_effect(
        t,
        None,
        P0,
        Effect::RemoveCounters {
            what: Sel::Target(0),
            kind: Some(kind.into()),
            n: Value::c(n),
        },
        &[Entity::Object(id)],
    );
}

/// Destroys a permanent (as an effect of a player 1 spell would).
pub fn destroy(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    run_effect(
        t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(id)],
    );
}

/// Triggered abilities on the stack whose text is `text`, with their sources.
pub fn stack_triggers(t: &TestGame, text: &str) -> Vec<(ObjectId, ObjectId)> {
    t.g.stack
        .iter()
        .copied()
        .filter_map(|id| match &t.g.obj(id).stack.as_ref()?.kind {
            StackKind::Triggered { ability, source } if ability.text.as_str() == text => {
                Some((id, *source))
            }
            _ => None,
        })
        .collect()
}

/// Names of the cards in a player's hand.
pub fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .hand
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// Names of the cards in a player's graveyard.
pub fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// Puts a real card on top of a player's library.
pub fn on_top(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.library_top(p, name)
}

/// The yes/no questions asked of `p` so far whose prompt contains `text`.
pub fn yes_no_asked(t: &TestGame, p: PlayerId, text: &str) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| {
            *q == p && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains(text))
        })
        .count()
}

/// Queues answers for `p`'s next replacement-effect choice (by option index).
pub fn choose_replacement(t: &mut TestGame, p: PlayerId, index: usize) {
    t.answer(p, DecisionKind::Replacement, Answer::Index(index));
}
