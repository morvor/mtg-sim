//! Shared helpers for rulings batch P030 (`r_p030_*.rs`): copies of spells (CR 707.10) and
//! tokens that are copies of the permanent that made them (CR 707.2, 111.4).

#![allow(dead_code)]

use crate::r_s25_common::cast_new;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The number of 1/1 red Elemental tokens `p` controls (made by Young Pyromancer:
/// "Whenever you cast an instant or sorcery spell, create a 1/1 red Elemental creature
/// token."), i.e. how many instant and sorcery spells `p` cast.
pub fn elementals(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| {
            o.controller == p && o.is_token() && o.chars.subtypes.iter().any(|s| s == "Elemental")
        })
        .count()
}

/// `p` casts Fling ("As an additional cost to cast this spell, sacrifice a creature. Fling
/// deals damage equal to the sacrificed creature's power to any target.") sacrificing a
/// fresh Hill Giant (3/3) and targeting `target`. Returns the spell.
pub fn fling_giant(t: &mut TestGame, p: PlayerId, target: Entity) -> ObjectId {
    let giant = t.battlefield(p, "Hill Giant");
    t.answer_choose(p, &[Entity::Object(giant)]);
    let fling = cast_new(t, p, "Fling", &[target]);
    assert!(!t.on_battlefield(giant), "the Giant was sacrificed to cast Fling");
    fling
}

/// The tokens `p` controls named `name`.
pub fn tokens_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Whether any copy of a spell is on the stack.
pub fn copy_on_stack(t: &TestGame) -> bool {
    t.g.stack
        .iter()
        .any(|id| t.g.obj(*id).kind == ObjKind::SpellCopy)
}

/// `p` surveils `n` (as a resolving effect would), then settles triggers.
pub fn surveil(t: &mut TestGame, p: PlayerId, n: i32) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::Surveil {
            who: PlayerRef::You,
            n: Value::c(n),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

/// `p` draws `n` cards (as a resolving effect would), then settles triggers.
pub fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

/// Moves the permanent to its owner's graveyard (destroying it) and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

/// Resolves the top of the stack until a copy of a spell is on it (or the stack is empty);
/// returns the copies, bottom first.
pub fn resolve_until_copies(t: &mut TestGame) -> Vec<ObjectId> {
    t.settle();
    for _ in 0..20 {
        let copies = crate::r_s25_common::spell_copies(t);
        if !copies.is_empty() || t.g.stack.is_empty() {
            return copies;
        }
        t.resolve();
    }
    crate::r_s25_common::spell_copies(t)
}
