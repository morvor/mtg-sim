//! Shared helpers for the tests of rulings batch P210 (`r_p210_*.rs`): enchant.

#![allow(dead_code)]

use crate::r_s01_common::*;
use crate::r_s02_common::can_activate;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::attach_new;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `source` deals `n` damage to `to` (a player or permanent) as a resolving effect would,
/// then state-based actions and triggers are settled.
pub fn deal(t: &mut TestGame, source: ObjectId, n: i32, to: impl Into<Entity>) {
    use mtg_engine::ability::{Effect, Sel, Value};
    let to = match to.into() {
        Entity::Object(o) => Entity::Object(t.g.current(o)),
        e => e,
    };
    let source = t.g.current(source);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), t.g.obj(source).controller);
    ctx.targets = vec![vec![to]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

/// Plenty of mana of every color for `p`.
pub fn lots_of_mana(t: &mut TestGame, p: PlayerId) {
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        add_mana(t, p, ty, 3);
    }
    add_mana(t, p, ManaType::C, 8);
}

/// P0 controls the real Aura `aura` on P1's `creature`. Whether (P0, P1) could activate
/// an activated ability of the creature (`on_aura == false`) or of the Aura, each in their
/// own precombat main phase with plenty of mana.
pub fn who_can_activate(aura: &str, creature: &str, on_aura: bool) -> (bool, bool) {
    supported(aura);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P1, creature);
    let a = attach_new(&mut t, P0, aura, c);
    let source = if on_aura { a } else { c };
    let mut out = [false, false];
    for (i, p) in [P0, P1].into_iter().enumerate() {
        t.set_step(p, Step::PrecombatMain);
        lots_of_mana(&mut t, p);
        out[i] = can_activate(&mut t, p, source);
    }
    (out[0], out[1])
}
