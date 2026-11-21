//! Shared helpers for the tests of rulings batch P131 (`r_p131_*.rs`), the "refund" group:
//! regeneration shields, abilities that add mana or return cards from a graveyard, and
//! spells whose targets become illegal. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p107_common::{act, hand_names, mana, pool_total};
pub use crate::r_p108_common::{obj, put_counters, resolved, tokens_with};
pub use crate::r_p130_common::{is_trigger, loyalty, top};
pub use crate::r_p133_common::{fizzles, kill, snap};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, give_mana_for, stack_library, supported, tokens,
};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy, target_candidates};
pub use crate::r_s03_common::{in_hand_with_mana, to_blockers};
pub use crate::r_s06_common::{activate_containing, attach_new, damage};
pub use crate::r_s25_common::{cast_new, lands_for_cost, spell_copies, targets_of};

/// The permanent was regenerated: still on the battlefield (the same object), tapped,
/// with no damage marked on it.
pub fn regenerated(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_live(id) && t.on_battlefield(id) && t.obj(id).tapped && t.obj(id).damage == 0
}

/// Activates `source`'s ability containing "Regenerate" and resolves the stack.
pub fn shield(t: &mut TestGame, p: PlayerId, source: ObjectId) {
    activate_containing(t, p, source, "egenerate").expect("regeneration not activatable");
    t.resolve_all();
}

/// Casts the real card `name` from `p`'s hand (lands for its mana cost are added) with the
/// given targets, one per target slot.
pub fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    lands_for_cost(t, p, name);
    let c = t.hand(p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"))
}

/// The card named `name` in `p`'s graveyard (the first one).
pub fn in_yard(t: &TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.g.find_in_zone(object::Zone::Graveyard(p), name)[0]
}

/// `p` answers every "Choose a creature type" choice with Vampire.
pub fn choose_vampire(t: &mut TestGame, p: PlayerId) {
    use mtg_engine::decision::{Answer, Decision};
    crate::r_s03_common::respond(t, p, |_, d| match d {
        Decision::ChooseOption {
            prompt, options, ..
        } if prompt == "Choose a creature type" => options
            .iter()
            .position(|o| o == "Vampire")
            .map(Answer::Index),
        _ => None,
    });
}
