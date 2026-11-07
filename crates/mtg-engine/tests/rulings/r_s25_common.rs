//! Shared helpers for the tests of rulings batch S25 (`r_s25_*.rs`): copies of spells and
//! abilities on the stack (CR 707.10), casting copies of cards (CR 707.12), and tokens that
//! are copies of permanents (CR 707.2, 111.4). (The helpers of batches S01–S24 are used
//! too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::object::{ObjKind, StackKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The targets chosen for the spell or ability `id` on the stack (every mode and target
/// slot, in order).
pub fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// The copies of spells on the stack, bottom first.
pub fn spell_copies(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy)
        .collect()
}

/// The abilities on the stack whose source is `source`, bottom first.
pub fn abilities_from(t: &TestGame, source: ObjectId) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            matches!(
                t.g.obj(*id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Activated { source: s, .. } | StackKind::Triggered { source: s, .. })
                    if *s == source
            )
        })
        .collect()
}

/// The value of X of the spell or ability `id` on the stack.
pub fn x_of(t: &TestGame, id: ObjectId) -> Option<i32> {
    t.g.obj(id).stack.as_deref().and_then(|si| si.x)
}

/// Queues `p`'s answers for a copy: choose new targets, changing its targets in order to
/// `new` (`None` keeps that target).
pub fn change_copy_targets(t: &mut TestGame, p: PlayerId, new: &[Option<Entity>]) {
    t.answer_yes(p, true);
    for e in new {
        t.answer(
            p,
            DecisionKind::Targets,
            Answer::Entities(e.iter().copied().collect()),
        );
    }
}

/// Queues `p`'s answer for a copy: keep its targets.
pub fn keep_copy_targets(t: &mut TestGame, p: PlayerId) {
    t.answer_yes(p, false);
}

/// Basic lands (Wastes for generic and colorless mana) to pay the mana cost `cost` of a
/// card's front face, paying each hybrid or Phyrexian symbol with its first color. {X} is
/// left out.
pub fn lands_for_cost(t: &mut TestGame, p: PlayerId, name: &str) {
    let c = mtg_engine::card::card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    for sym in format!("{cost}")
        .split('}')
        .filter_map(|s| s.strip_prefix('{'))
    {
        let land = match sym.chars().next() {
            Some('W') => "Plains",
            Some('U') => "Island",
            Some('B') => "Swamp",
            Some('R') => "Mountain",
            Some('G') => "Forest",
            Some('X') => continue,
            _ => {
                let n = sym.parse::<usize>().unwrap_or(1);
                t.lands(p, "Wastes", n);
                continue;
            }
        };
        t.lands(p, land, 1);
    }
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost, then casts it
/// with the given targets (one per target slot). Returns the spell.
pub fn cast_new(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// The number of creature tokens `p` controls.
pub fn creature_tokens(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.is(CardType::Creature))
        .count()
}

/// The name of the object (followed across zone changes) now.
pub fn name_now(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// Whether the object (followed across zone changes) is legendary now.
pub fn legendary(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.supertypes.contains(Supertype::Legendary)
}
