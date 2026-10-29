//! Shared helpers for the tests of rulings batch P147 (`r_p147_*.rs`): repeatable
//! Treasures, "return a creature you control" engines, restocking graveyards onto the
//! library, and creatures that can't attack or block (alone, unless ...). (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p079_common::enter_attacking;
pub use crate::r_p108_common::{end_step, resolved, two_headed_giant};
pub use crate::r_p141_common::{hand_names, now, offered_target};
pub use crate::r_p146_common::{
    counter, destroy_together, enter_together, fill_hand, n_asked, stacked_triggers, treasures,
};
pub use crate::r_p148_common::{at, declare_blocks, legal_team_blocks, mana, provoke};
pub use crate::r_p160_common::{activate_resolve, cast_resolve, yes};
pub use crate::r_p057_common::{into_upkeep, pump};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, place, stack_library, supported, tokens,
    triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_cast, destroy, target_candidates};
pub use crate::r_s05_common::{enter, move_to, run_from};
pub use crate::r_s09_common::{legal_attack, to_combat};
pub use crate::r_s10_common::{attacking, blocking};
pub use crate::r_s21_common::{enter_blocking, legal_blocks};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` gains control of `id` indefinitely (as a resolving "gain control of target
/// creature" would).
pub fn take_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
}

/// The names of the cards in `p`'s library, top first.
pub fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// The name of the top card of `p`'s library.
pub fn top_name(t: &TestGame, p: PlayerId) -> String {
    library_names(t, p).first().cloned().unwrap_or_default()
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost, then casts it
/// choosing all of `targets` for its first target slot ("any number of target ...").
pub fn cast_multi(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    t.cast(p, card).targets(targets).go()
}
