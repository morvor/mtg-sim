//! Shared helpers for the tests of rulings batch S19 (`r_s19_*.rs`): the Class, flip, meld,
//! modal double-faced, plane, Saga, and scheme layouts. (The helpers of batches S01–S16
//! are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::casting::Illegal;
use mtg_engine::classes;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The uid of the Class `class`'s level bar that advances it to level `n` (CR 716.2a).
pub fn level_bar(t: &mut TestGame, class: ObjectId, n: u32) -> u64 {
    t.g.recompute();
    t.obj_now(class)
        .chars
        .abilities
        .iter()
        .find(|a| {
            matches!(&a.kind, AbilityKind::Activated(act)
                if matches!(act.body.effect, Effect::SetClassLevel { level } if level == n))
        })
        .map(|a| a.uid)
        .expect("class level bar")
}

/// `p` activates the level bar of `class` that advances it to level `n` (paying with
/// whatever mana `p` has); the ability is left on the stack.
pub fn gain_level(t: &mut TestGame, p: PlayerId, class: ObjectId, n: u32) -> Result<(), Illegal> {
    let uid = level_bar(t, class, n);
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, t.g.current(class), uid).map(|_| ());
    t.g.flush_events();
    r
}

/// The level of the permanent `id` (CR 716.2d: one without a level is level 1).
pub fn level(t: &TestGame, id: ObjectId) -> u32 {
    classes::level(&t.g, t.g.current(id))
}

/// Puts `n` lore counters on the Saga `id` as an effect would, then puts the chapter
/// abilities that triggered on the stack.
pub fn add_lore(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::LORE, n, None);
    t.g.flush_events();
    t.settle();
}

/// Removes `n` lore counters from the Saga `id` as an effect would.
pub fn remove_lore(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.remove_counters(Entity::Object(id), counters::LORE, n);
    t.g.flush_events();
    t.settle();
}

/// The lore counters on the Saga `id`.
pub fn lore(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::LORE)
}

/// The chapter numbers of the chapter abilities on the stack whose source is `saga`
/// (bottom of the stack first). A chapter symbol list ("I, II —") is one ability that
/// triggers once for each of its chapter numbers (CR 714.2c).
pub fn chapters_on_stack(t: &TestGame, saga: ObjectId) -> Vec<u32> {
    t.g.stack
        .iter()
        .filter_map(|id| {
            let si = t.g.obj(*id).stack.as_ref()?;
            match &si.kind {
                StackKind::Triggered { source, ability } if *source == saga => {
                    let AbilityKind::Triggered(tr) = &ability.kind else {
                        return None;
                    };
                    let TriggerCond::Custom(name) = &tr.trigger else {
                        return None;
                    };
                    name.starts_with("chapter:")
                        .then(|| si.event.as_ref().map(|e| e.amount as u32))
                        .flatten()
                }
                _ => None,
            }
        })
        .collect()
}
