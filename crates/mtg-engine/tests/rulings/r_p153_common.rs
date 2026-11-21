//! Shared helpers for the tests of rulings batch P153 (`r_p153_*.rs`): abilities that
//! scale with a mana value. (The helpers of batches S01–S09 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Adds `n` mana of type `ty` to `p`'s pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// `p` casts Blaze ({X}{R}: X damage to any target) with the given X at `target`, paying
/// from their pool; the spell's id.
pub fn cast_blaze(t: &mut TestGame, p: PlayerId, x: i64, target: Entity) -> ObjectId {
    let blaze = t.hand(p, "Blaze");
    mana(t, p, ManaType::R, x as u32 + 1);
    t.g.turn.priority = Some(p);
    t.cast(p, blaze).x(x).target(target).go()
}

/// `p` casts the real card `name` from their hand with no targets, paying from a pool
/// filled with the right mana; the spell's id.
pub fn cast_simple(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let c = t.hand(p, name);
    mana_for(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast(p, c).go()
}

/// Fills `p`'s pool with the mana cost of the real card `name` (X as 0).
pub fn mana_for(t: &mut TestGame, p: PlayerId, name: &str) {
    let c = mtg_engine::card::card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let ty = match sym {
            "W" => ManaType::W,
            "U" => ManaType::U,
            "B" => ManaType::B,
            "R" => ManaType::R,
            "G" => ManaType::G,
            "X" => continue,
            n => {
                let n = n.parse::<u32>().unwrap_or(1);
                mana(t, p, ManaType::C, n);
                continue;
            }
        };
        mana(t, p, ty, 1);
    }
}

/// `p` counters `spell` with Counterspell (paid from their pool) and resolves the
/// Counterspell (only it: the rest of the stack stays).
pub fn counter(t: &mut TestGame, p: PlayerId, spell: ObjectId) {
    let cs = t.hand(p, "Counterspell");
    mana(t, p, ManaType::U, 2);
    t.g.turn.priority = Some(p);
    t.cast(p, cs).target(spell).go();
    t.settle();
    t.g.resolve_top();
    t.settle();
    assert_eq!(t.zone(spell), Zone::Graveyard(t.g.obj(spell).owner));
}

/// The mana value of the object (followed across zone changes) now.
pub fn mv(t: &TestGame, id: ObjectId) -> u32 {
    t.g.mana_value_of(t.g.current(id))
}

/// The sources (as ids) of the triggered abilities on the stack whose text contains
/// `text`.
pub fn trigger_sources(t: &TestGame, text: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .filter_map(|id| {
            let si = t.g.obj(*id).stack.as_ref()?;
            match &si.kind {
                StackKind::Triggered { ability, source, .. } if ability.text.contains(text) => {
                    Some(*source)
                }
                _ => None,
            }
        })
        .collect()
}

/// Answers X with `x` for `p`'s next X choice.
pub fn answer_x(t: &mut TestGame, p: PlayerId, x: i64) {
    t.answer(p, DecisionKind::X, Answer::Number(x));
}

/// Number of X choices asked of `p` since decision `from`.
pub fn x_asked(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, mtg_engine::decision::Decision::ChooseX { .. }))
        .count()
}

/// The candidates of the entity choices (non-target) asked of `p` since decision `from`.
pub fn entity_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } if *q == p => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect()
}

/// Puts `p`'s whole library to `names` (top first), removing the filler cards.
pub fn set_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    let old = std::mem::take(&mut t.g.players[p.idx()].library);
    for id in old {
        t.g.objects[id.0 as usize].zone = Zone::Nowhere;
    }
    crate::r_s01_common::stack_library(t, p, top_first)
}
