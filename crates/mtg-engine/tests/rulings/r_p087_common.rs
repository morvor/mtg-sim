//! Shared helpers for the tests of rulings batch P087 (`r_p087_*.rs`): hate cards (cards
//! that punish or restrict a mechanic, a card type, tapped creatures, tokens, tutors, a
//! creature type or a color), spell-count limits ("can't cast more than one spell each
//! turn") and hatebears. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_s01_common::{supported, watch};
pub use crate::r_s02_common::destroy;
pub use crate::r_s03_common::{in_hand_with_mana, run_effect};
pub use crate::r_s04_common::{ability_targets, next_upkeep, spell_targets};
pub use crate::r_s06_common::give_control;
pub use crate::r_s21_common::castable;
pub use crate::r_p108_common::put_counters;
pub use crate::r_s01_common::{attack_with, block_and_finish};
pub use crate::r_s02_common::create_token;
pub use crate::r_s03_common::to_blockers;
pub use crate::r_s04_common::add_mana;
pub use crate::r_s06_common::{activate_containing, attach_new, attached_to};
pub use crate::r_s17_common::token_copy;
pub use crate::r_s20_common::tap_for_mana;
pub use crate::r_s21_common::{blocks_now, legal_blocks};
pub use crate::r_s25_common::{cast_new, lands_for_cost};
pub use crate::r_s29_common::damage_marked;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether `p` could cast a copy of the card `name` put into their hand now, given the
/// lands to pay its mana cost (the card stays in their hand).
pub fn could_cast(t: &mut TestGame, p: PlayerId, name: &str) -> bool {
    let c = in_hand_with_mana(t, p, name);
    castable(t, p, c)
}

/// `p` activates the ability of `source` whose text contains `needle` (with the given
/// target answers queued), panicking if that's illegal; the ability stays on the stack.
pub fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str, targets: &[Entity]) {
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    activate_containing(t, p, source, needle)
        .unwrap_or_else(|e| panic!("couldn't activate {needle:?}: {e:?}"));
    t.settle();
}

/// Taps the permanent (an effect tapping it) and settles: triggers go on the stack.
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}

/// Untaps the permanent and settles.
pub fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.flush_events();
    t.settle();
}

/// Sets the loyalty of the planeswalker.
pub fn set_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), n);
}

/// Whether the object (followed across zone changes) is tapped.
pub fn is_tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Every candidate offered by the object choices (`ChooseEntities`) asked since decision
/// `from`.
pub fn entity_candidates(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .flatten()
        .collect()
}
