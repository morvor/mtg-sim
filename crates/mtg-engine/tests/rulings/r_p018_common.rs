//! Shared helpers for the tests of rulings batch P018 (`r_p018_*.rs`): lands that become
//! creatures, color changes, "if {C} was spent", damage triggers, phasing, and "discard
//! two cards unless you discard ...". (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use mtg_engine::ability::{Duration, Effect, Modification, Sel, Value};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Fills `p`'s mana pool with plenty of mana of every type.
pub fn plenty_of_mana(t: &mut TestGame, p: PlayerId) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        add_mana(t, p, ty, 5);
    }
}

/// Empties `p`'s mana pool.
pub fn empty_pool(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].mana_pool.mana.clear();
}

/// `p` activates the activated ability of `source` whose text contains `needle` (paying
/// from a pool full of mana, emptied afterwards), and it resolves.
pub fn activate_and_resolve(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    plenty_of_mana(t, p);
    activate_containing(t, p, source, needle).expect("activate");
    empty_pool(t, p);
    t.resolve();
}

/// Applies `mods` to the permanent until end of turn, as a resolving spell controlled by
/// the permanent's controller would (CR 611.2), and settles.
pub fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    let id = t.g.current(id);
    let mut ctx = mtg_engine::eval::Ctx::new(None, t.obj(id).controller);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.settle();
}

/// Makes the permanent a 0/0 creature in addition to its other types until end of turn
/// (as an awaken spell's "becomes a 0/0 Elemental creature" would).
pub fn make_zero_zero_creature(t: &mut TestGame, id: ObjectId) {
    modify(
        t,
        id,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::Const(0)), Some(Value::Const(0))),
        ],
    );
}

/// Puts `n` counters of `kind` on the permanent.
pub fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}

/// Whether the permanent (followed across zone changes) is a creature now.
pub fn is_creature_now(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).is(CardType::Creature)
}

/// The colors of the object (followed across zone changes) now.
pub fn colors_now(t: &mut TestGame, id: ObjectId) -> ColorSet {
    t.g.recompute();
    t.obj_now(id).chars.colors
}
