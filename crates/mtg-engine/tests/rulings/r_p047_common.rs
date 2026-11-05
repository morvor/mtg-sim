//! Shared helpers for the tests of rulings batch P047 (`r_p047_*.rs`): disenchant
//! effects, "if it would die this turn, exile it instead", wheels and hand dumps, and
//! the Baldur's Gate characters.

#![allow(dead_code)]

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Puts `name` into `p`'s hand, gives `p` lands for its mana cost (plus `extra` Wastes
/// for X), and casts it with one answer per target slot. Returns the spell.
pub fn cast_x(t: &mut TestGame, p: PlayerId, name: &str, x: Option<i64>, targets: &[Entity]) -> ObjectId {
    supported(name);
    let c = t.hand(p, name);
    give_mana_for(t, p, name);
    if let Some(x) = x {
        t.lands(p, "Wastes", x.max(0) as usize);
    }
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let mut b = t.cast(p, c);
    if let Some(x) = x {
        b = b.x(x);
    }
    b.go()
}

/// Casts `name` (see [`cast_x`]) and resolves the stack.
pub fn spell(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let id = cast_x(t, p, name, None, targets);
    t.resolve_all();
    id
}

/// Casts an X spell and resolves the stack.
pub fn spell_x(t: &mut TestGame, p: PlayerId, name: &str, x: i64, targets: &[Entity]) -> ObjectId {
    let id = cast_x(t, p, name, Some(x), targets);
    t.resolve_all();
    id
}

/// Destroys `id` with Murder cast by `p`.
pub fn murder(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    spell(t, p, "Murder", &[obj(id)]);
}

/// Cards in `p`'s hand.
pub fn hand_cards(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.players[p.idx()].hand.clone()
}

/// Fills `p`'s hand with `n` copies of `name`.
pub fn fill_hand(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.hand(p, name)).collect()
}

/// Puts `id` into its owner's graveyard (not a discard, not dying).
pub fn to_graveyard(t: &mut TestGame, id: ObjectId) {
    let owner = t.g.obj(id).owner;
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Graveyard(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
}
