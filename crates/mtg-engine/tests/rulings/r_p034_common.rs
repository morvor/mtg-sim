//! Shared helpers for the tests of rulings batch P034 (`r_p034_*.rs`): "counters matter"
//! cards and cards that use counters as fuel (counters removed as a cost, counted on
//! resolution, preserved when the permanent dies). The helpers of batches S01 and P038
//! are used too.

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` counters of `kind` on the object (followed across zone changes) or player,
/// as an effect would, and settles.
pub fn put(t: &mut TestGame, e: impl Into<Entity>, kind: &str, n: u32) {
    let e = match e.into() {
        Entity::Object(id) => Entity::Object(t.g.current(id)),
        other => other,
    };
    t.g.add_counters(e, kind, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn pool(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// `p` casts the card `name` from hand (mana for its cost given) with the given targets,
/// leaving it on the stack with any cast triggers above it.
pub fn cast_card(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let s = crate::r_p038_common::cast_only(t, p, name, targets);
    t.settle();
    s
}

/// P1 casts Cancel at the spell `spell` (with exactly {1}{U}{U}), and Cancel resolves.
pub fn opponent_counters(t: &mut TestGame, spell: ObjectId) {
    let c = cast_card(t, P1, "Cancel", &[Entity::Object(spell)]);
    // Cancel is on top of the stack: resolve just it.
    assert_eq!(t.g.stack.last(), Some(&c));
    t.resolve();
    assert!(!t.g.stack.contains(&spell), "the spell was countered");
}

/// The top object of the stack is a triggered ability (not a spell).
pub fn top_is_trigger(t: &TestGame) -> bool {
    t.g.stack.last().is_some_and(|id| {
        t.g.obj(*id)
            .stack
            .as_ref()
            .is_some_and(|si| matches!(si.kind, object::StackKind::Triggered { .. }))
    })
}

/// The damage events of this turn as (source, target, amount).
pub fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

/// Total number of counters (of every kind) on the object.
pub fn total_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counters.values().sum()
}

/// `p` casts Give (the left half of Give // Take: "Put three +1/+1 counters on target
/// creature") at `target`, and it resolves.
pub fn give(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    pool(t, p, ManaType::G, 1);
    pool(t, p, ManaType::C, 2);
    let c = t.hand(p, "Give // Take");
    t.cast(p, c)
        .method(mtg_engine::object::CastMethod::Half(0))
        .target(target)
        .go();
    t.resolve_all();
}
