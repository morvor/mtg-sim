//! Shared helpers for the tests of rulings batch P169 (`r_p169_*.rs`): "power 2 or
//! less", graveyard-hate and mill payoffs, modified creatures, Mountain matters,
//! multicolored and noncreature spell payoffs. (The helpers of earlier batches are used
//! too.)

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p076_common::{is_blocked, mana};
pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_p116_common::{enter_together, set_life};
pub use crate::r_p119_common::{counter_spell, enter_with, has_kw};
pub use crate::r_p189_common::{stack_triggers_from, triggers_of};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_attack, can_cast, destroy};
pub use crate::r_s21_common::{castable, go_to, legal_blocks};
pub use crate::r_s25_common::{
    abilities_from, cast_new, creature_tokens, lands_for_cost, spell_copies, targets_of,
};

/// `p` casts Opt (adding {U} first) and returns the spell.
pub fn cast_opt(t: &mut TestGame, p: PlayerId) -> ObjectId {
    mana(t, p, ManaType::U, 1);
    let opt = t.hand(p, "Opt");
    t.cast(p, opt).go()
}

/// P1 casts Counterspell targeting `spell` and it resolves (only Counterspell).
pub fn counterspell(t: &mut TestGame, spell: ObjectId) {
    mana(t, P1, ManaType::U, 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(!t.g.stack.contains(&spell), "the spell wasn't countered");
}

/// The triggered abilities on the stack, bottom first.
pub fn stack_triggers(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            t.g.obj(*id)
                .stack
                .as_ref()
                .is_some_and(|si| matches!(si.kind, StackKind::Triggered { .. }))
        })
        .collect()
}

/// Damage marked on the object (followed across zone changes).
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
