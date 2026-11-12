//! Shared helpers for the CR 603 audit tests (`r603_audit_*.rs`): card support checks,
//! what is on the stack (which triggered abilities, whose), and small setup shortcuts.

#![allow(dead_code)]

use mtg_engine::card::card;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts the compiler fully supports `name` (a test on a half-supported card would
/// test the wrong thing).
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// The triggered abilities on the stack, bottom first: (source, controller, source name).
pub fn stack_triggers(t: &TestGame) -> Vec<(ObjectId, PlayerId, String)> {
    t.g.stack
        .iter()
        .filter_map(|s| {
            let o = t.g.obj(*s);
            match &o.stack.as_ref()?.kind {
                StackKind::Triggered { source, .. } => Some((
                    *source,
                    o.controller,
                    t.g.obj(*source).chars.name.to_string(),
                )),
                _ => None,
            }
        })
        .collect()
}

/// How many triggered abilities of sources named `name` are on the stack.
pub fn triggers_of(t: &TestGame, name: &str) -> usize {
    stack_triggers(t).iter().filter(|x| x.2 == name).count()
}

/// The controllers of the triggered abilities of sources named `name` on the stack,
/// bottom first.
pub fn trigger_controllers(t: &TestGame, name: &str) -> Vec<PlayerId> {
    stack_triggers(t)
        .into_iter()
        .filter(|x| x.2 == name)
        .map(|x| x.1)
        .collect()
}

/// Queues `n` target answers naming `target` for `p` (one per triggered ability).
pub fn targets_n(t: &mut TestGame, p: PlayerId, target: Entity, n: usize) {
    for _ in 0..n {
        t.answer_targets(p, &[target]);
    }
}

/// The permanents named `name` that `p` controls.
pub fn controlled_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.g.obj(*id).controller == p)
        .count()
}

/// The tokens `p` controls whose type gives them the name `kind` ("Spirit" for the
/// "Spirit Token" a Spirit creature token is named, CR 111.4).
pub fn tokens_named(t: &TestGame, p: PlayerId, kind: &str) -> usize {
    controlled_named(t, p, &format!("{kind} Token"))
}
