//! Shared helpers for the tests of item `rulings-residue-misc-2`, pass A
//! (`r_residue_misc_2a_*.rs`): the open rulings left in batches P117, P122, P124–P126,
//! P133, P134, P136, P137, P141–P143, P145, P146, P148, P150 and P151. The helpers of
//! earlier batches are used too.

#![allow(dead_code)]

pub use crate::r_p007_common::{dmg, obj, pl};
pub use crate::r_p023_common::clone_of;
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, stack_library, supported, tokens,
};
pub use crate::r_s02_common::{can_activate, can_attack, can_cast, destroy};
pub use crate::r_s06_common::{activate_containing, give_control};
pub use crate::r_s21_common::legal_blocks;
pub use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts `n` +1/+1 counters on the permanent (as a resolving effect would).
pub fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, n, None);
    t.g.recompute();
    t.g.flush_events();
}

/// Moves the card (already in `p`'s library) to the bottom of that library.
pub fn to_bottom(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    let lib = &mut t.g.players[p.idx()].library;
    lib.retain(|id| *id != card);
    lib.insert(0, card);
}

/// The controller of the object (followed across zone changes) now.
pub fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// The number of abilities the object (followed across zone changes) has now.
pub fn n_abilities(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id).chars.abilities.len()
}

/// Everyone passes until the stack is empty, then the state settles.
pub fn settle_all(t: &mut TestGame) {
    t.resolve_all();
    t.settle();
}

pub use crate::r_p079_common::{add, back_face};

/// The source deals `n` damage to `to` (as a resolving effect of it would), then the
/// state settles.
pub fn damage_to(t: &mut TestGame, source: ObjectId, n: i32, to: Entity) {
    use mtg_engine::ability::{Effect, Sel, Value};
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
