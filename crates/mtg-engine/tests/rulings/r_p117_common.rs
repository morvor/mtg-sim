//! Shared helpers for the tests of rulings batch P117 (`r_p117_*.rs`): "power matters"
//! abilities — a source's own power (last known information, negative power, values
//! fixed on resolution) and comparisons of powers among creatures. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, resolved};
pub use crate::r_p116_common::{cast_resolve, n_asked, set_life};
pub use crate::r_p119_common::{
    counter_spell, enter_with, giant_growth, pool_total, stack, with_counters, wrath,
};
pub use crate::r_p120_common::{gain_life, give_minus1, give_plus1, plus1, tokens_named};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, custom_card, stack_library, supported, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_attack, can_cast, destroy};
pub use crate::r_s24_common::enter_together;
pub use crate::r_s25_common::{cast_new, creature_tokens, lands_for_cost};

/// Exiles a permanent (or any object, followed across zone changes) and settles.
pub fn exile(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// In response to an ability of `src`: it gets `n` +1/+1 counters, then is exiled, so
/// its power as it last existed on the battlefield differs from its printed power.
pub fn grow_then_exile(t: &mut TestGame, src: ObjectId, n: u32) {
    give_plus1(t, src, n);
    exile(t, src);
    assert!(!t.on_battlefield(src));
}

/// Damage marked on a permanent.
pub fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.g.obj(t.g.current(id)).damage
}

/// The life `p` has lost relative to 20.
pub fn lost(t: &TestGame, p: PlayerId) -> i32 {
    20 - t.life(p)
}

/// Advances to P0's beginning of combat step, its triggers on the stack.
pub fn beginning_of_combat(t: &mut TestGame) {
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.settle();
}

/// Applies `mods` to the permanent until end of turn, then settles.
pub fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<mtg_engine::ability::Modification>) {
    use mtg_engine::ability::{Duration, Effect, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Gives `id` -n/-0 until end of turn.
pub fn minus_power(t: &mut TestGame, id: ObjectId, n: i32) {
    use mtg_engine::ability::{Modification, Value};
    modify(t, id, vec![Modification::ModifyPT(Value::c(-n), Value::c(0))]);
}

/// `p` gains control of the permanent until end of turn, then settles.
pub fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}
