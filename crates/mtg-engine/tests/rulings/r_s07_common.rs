//! Shared helpers for the tests of rulings batch S07 (`r_s07_*.rs`): escalate, escape,
//! eternalize, evoke, evolve, exalted, exert, exhaust, exploit, explore, extort,
//! fabricate, fathomless descent. (The helpers of batches S01–S05 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the spell `spell` resolved this turn (it wasn't countered and didn't fail to
/// resolve for lack of legal targets).
pub fn resolved(t: &TestGame, spell: ObjectId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::SpellResolved { spell: s } if *s == spell))
}

/// Number of decisions matching `pred` asked of anyone since decision `from`.
pub fn count_asked(t: &TestGame, from: usize, pred: impl Fn(&Decision) -> bool) -> usize {
    t.asked()[from..].iter().filter(|(_, d)| pred(d)).count()
}

/// Whether the decision is a priority decision.
pub fn is_priority(d: &Decision) -> bool {
    matches!(d, Decision::Priority { .. })
}

/// Number of untapped permanents named `name` controlled by `p`.
pub fn untapped_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name && !o.tapped)
        .count()
}

/// The damage marked on the object (followed across zone changes).
pub fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// Creatures on the battlefield controlled by `p` with the given name.
pub fn named_of(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has_kw(t: &TestGame, id: ObjectId, kw: mtg_engine::keywords::KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(kw)
}

/// Whether `id` (followed across zone changes) is a creature on the battlefield.
pub fn creature_on_battlefield(t: &TestGame, id: ObjectId) -> bool {
    t.on_battlefield(id) && t.obj_now(id).is(CardType::Creature)
}

/// The modes chosen for the spell or ability `id` on the stack.
pub fn chosen_modes(t: &TestGame, id: ObjectId) -> Vec<usize> {
    t.g.obj(id)
        .stack
        .as_ref()
        .map(|si| si.chosen.iter().filter_map(|c| c.mode).collect())
        .unwrap_or_default()
}
