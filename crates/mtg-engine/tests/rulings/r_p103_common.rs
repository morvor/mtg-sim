//! Shared helpers for the tests of rulings batch P103 (`r_p103_*.rs`): milling any player
//! (counterspells that mill, "mills half their library", reveal-until-lands, mill
//! triggers), exiling from libraries, and a few "meme" cards (Panglacial Wurm, Twiddle,
//! Fact or Fiction, Asmoranomardicadaistinaculdacar).

#![allow(dead_code)]

use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Empties `p`'s library (exiling its cards), then puts `n` cards named `name` in it.
pub fn library_of(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    let names = vec![name; n];
    crate::r_p189_common::set_library(t, p, &names)
}

/// Empties `p`'s library (exiling its cards).
pub fn empty_library(t: &mut TestGame, p: PlayerId) {
    let lib = t.g.player(p).library.clone();
    for c in lib {
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, None);
    }
    t.g.flush_events();
}

/// The names of the cards in `p`'s graveyard, bottom first.
pub fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// The names of the cards in `p`'s library, top first.
pub fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// Whether the object `id` is on the stack now.
pub fn on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.g.stack.contains(&id)
}
