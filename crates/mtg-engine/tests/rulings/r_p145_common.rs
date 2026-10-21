//! Shared helpers for the tests of rulings batch P145 (`r_p145_*.rs`): repeatable looting
//! ("draw a card, then discard a card") engines and their interactions. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p057_common::into_beginning_of_combat;
pub use crate::r_p108_common::{end_step, obj, put_counters, resolved};
pub use crate::r_p143_common::{answer_once_when, asked_of, entity_choices, n_asked};
pub use crate::r_p146_common::{counter, destroy_together, discard, fill_hand, treasures};
pub use crate::r_p160_common::{activate_resolve, cast_resolve, mana, yes};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, stack_library, supported, tokens, triggers_on_stack,
    watch,
};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, create_token, destroy};
pub use crate::r_s05_common::{enter, move_to, run_from};
pub use crate::r_p057_common::into_upkeep;
pub use crate::r_p146_common::{enter_together, stacked_triggers};
pub use crate::r_s01_common::custom_card;
pub use crate::r_s04_common::add_mana;
pub use crate::r_s06_common::activate_containing;
pub use crate::r_s08_common::legal_cast_methods;
pub use crate::r_s17_common::name_of;
pub use crate::r_s25_common::{cast_new, lands_for_cost, spell_copies, targets_of};

/// Puts `n` counters of `kind` on the player `p` (as an effect would).
pub fn player_counters(t: &mut TestGame, p: PlayerId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Player(p), kind, n, None);
    t.g.flush_events();
    t.settle();
}

/// The counters of `kind` the player `p` has.
pub fn counters_on_player(t: &TestGame, p: PlayerId, kind: &str) -> u32 {
    t.g.player(p).counters.get(kind).copied().unwrap_or(0)
}

/// Whether `d` is the choice of the cards to discard.
pub fn is_discard(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("discard"))
}

/// Queues `p`'s answer to the next choice of cards to discard: `cards`.
pub fn discard_these(t: &mut TestGame, p: PlayerId, cards: &[ObjectId]) {
    t.answer(
        p,
        DecisionKind::Entities,
        Answer::Entities(cards.iter().map(|c| Entity::Object(*c)).collect()),
    );
}

/// The names of the cards in `p`'s hand, sorted.
pub fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .player(p)
        .hand
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// The names of the cards in `p`'s graveyard, sorted.
pub fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .player(p)
        .graveyard
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// Puts `n` filler cards into `p`'s graveyard.
pub fn fill_graveyard(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.graveyard(p, "Grizzly Bears");
    }
}
