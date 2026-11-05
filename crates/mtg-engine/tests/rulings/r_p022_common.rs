//! Shared helpers for rulings batch P022 (`r_p022_*.rs`): permanents that become or enter
//! as copies of creatures and effects that create token copies — what a copy copies (CR
//! 707.2, 707.3), the exceptions of copy effects (CR 707.9), "as enters" choices and
//! enters triggers of something that only becomes a copy (CR 603.6a, 607.5a, 614.12),
//! {X} in a copied mana cost (CR 107.3, 202.3e), and modal or multi-target copy spells
//! whose targets become illegal (CR 608.2b).

#![allow(dead_code)]

pub use crate::r_p021_common::{
    assert_printed_giant, cant_copy_what_enters_with_it, dress, dressed_giant, giant_growth,
    legendary, modify, untap,
};
pub use crate::r_p023_common::{hydra_cast_with_x3, is_wolf};
pub use crate::r_p024_common::*;
pub use crate::r_s01_common::{asked_since, stack_library};
pub use crate::r_p026_common::{activate_for_tokens, one};
pub use crate::r_s06_common::give_control;
pub use crate::r_s26_common::{dress_up, fresh};
pub use mtg_engine::object::Zone;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub const HYDRA: &str = "Benevolent Hydra";

/// Asserts `id` is a Benevolent Hydra ({X}{G}{G}, "enters with X +1/+1 counters") with
/// X = 0: mana value 2 and no counters.
pub fn hydra_with_x0(t: &mut TestGame, id: ObjectId) {
    assert_eq!(name_of(t, id), HYDRA);
    assert_eq!(mv_now(t, id), 2);
    assert_eq!(t.counters(id, counters::PLUS1), 0);
}

/// Queues `p`'s answer to a "choose a color" decision.
pub fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, mtg_engine::decision::Answer::Index(i));
}

/// Puts a -1/-1 counter on the permanent (as a resolving effect would).
pub fn minus1(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::MINUS1, 1, None);
    t.g.recompute();
    t.settle();
}

/// The index among the object's activated abilities of the first one whose text contains
/// `needle`.
pub fn activated_index(t: &TestGame, id: ObjectId, needle: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(needle))
        .unwrap_or_else(|| panic!("no activated ability containing {needle:?}"))
}

/// The permanents named `name` on the battlefield that `p` controls.
pub fn named_of(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj(*id).controller == p)
        .collect()
}
