//! Shared helpers for the tests of rulings batch P032 (`r_p032_*.rs`): permanents that
//! create token copies of themselves, spells that copy themselves, and cost increases and
//! reductions. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Tokens named `name` on the battlefield controlled by `p`.
pub fn tokens_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Number of abilities of the object whose text contains `s`.
pub fn abilities_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(s))
        .count()
}

/// Changes the permanent in every way a copy doesn't copy (tapped, two +1/+1 counters,
/// +3/+3 and green-blue until end of turn), plus 1 damage marked on it when `damage`.
pub fn dress(t: &mut TestGame, id: ObjectId, damage: bool) {
    dress_up(t, id);
    if damage {
        let id = t.g.current(id);
        t.g.obj_mut(id).damage += 1;
    }
}

/// Asserts that the token `id` is a plain copy of the printed `name`: its printed power
/// and toughness (when given), colors, untapped, no counters, no damage, nothing attached.
pub fn assert_plain_copy(t: &TestGame, id: ObjectId, name: &str, pt: Option<(i32, i32)>) {
    let o = t.obj_now(id);
    assert!(o.is_token(), "{name}: not a token");
    assert_eq!(o.chars.name, name);
    let printed = mtg_engine::card::card(name).front().chars.colors;
    assert_eq!(o.chars.colors, printed, "{name}: colors");
    if let Some(pt) = pt {
        assert_eq!(t.pt(id), pt, "{name}: power and toughness");
    }
    assert!(fresh(t, id), "{name}: token copied the tapped state, counters or attachments");
    assert_eq!(o.damage, 0, "{name}: token copied the damage");
}

/// Adds `n` mana of type `ty` to `p`'s pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: mtg_engine::mana::ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}
