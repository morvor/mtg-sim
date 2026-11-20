//! Shared helpers for the tests of item `rulings-residue-keywords`
//! (`r_residue_keywords_*.rs`): the open rulings left in batches P205, P214, P215, P217,
//! P222 and P226 — keyword leftovers (fight, landfall, imprint, transform, regenerate,
//! scry, Treasure, ...). (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p108_common::{obj, two_headed_giant};
pub use crate::r_p121_common::{cast_resolve, damage};
pub use crate::r_p135_common::{dealt, try_cast};
pub use crate::r_p205_common::{bury, delirium_graveyard, is_attacking};
pub(crate) use crate::r_p215_fight::make_illegal;
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, creatures, custom_card, stack_library, supported,
    tokens, triggers_on_stack, with_subtype,
};
pub use crate::r_s02_common::{can_activate, can_attack, can_cast, create_token, destroy};
pub use crate::r_s05_common::{move_to, run_from};
pub use crate::r_s07_common::{count_asked, damage_on, is_priority, resolved};
pub use crate::r_s24_common::{controller, enter_together, tapped};
pub use crate::r_s25_common::{cast_new, lands_for_cost, name_now};

/// Gives the object (followed across zone changes) the keyword until end of turn, as an
/// effect of its controller's.
pub fn grant_kw(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    let id = t.g.current(id);
    let c = t.obj_now(id).controller;
    run_from(
        t,
        c,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(kw))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has(t: &mut TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(kw)
}

/// The power and toughness of the object (followed across zone changes) now.
pub fn pt_now(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(t.g.current(id))
}

/// The number of +1/+1 counters on the object (followed across zone changes).
pub fn p1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// Whether any decision asked since `from` satisfies `pred`.
pub fn was_asked(t: &TestGame, from: usize, pred: impl Fn(&Decision) -> bool) -> bool {
    asked_since(t, from).iter().any(|(_, d)| pred(d))
}

/// The number of tokens `p` controls with the subtype.
pub fn tokens_of(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

/// The amount of mana of the type in `p`'s mana pool.
pub fn mana(t: &TestGame, p: PlayerId, ty: mtg_engine::mana::ManaType) -> u32 {
    crate::r_s24_common::pool(t, p, ty)
}
