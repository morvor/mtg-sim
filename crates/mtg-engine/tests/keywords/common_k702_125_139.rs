//! Shared helpers for the tests of CR 702.125–702.139 (undaunted, improvise, aftermath,
//! embalm, eternalize, afflict, ascend, assist, jump-start, mentor, afterlife, riot,
//! spectacle, escape, companion).

#![allow(dead_code, unused_imports)]

pub use crate::common_k702_011_017::{
    assert_supported, attack_with, bf, block_and_finish, custom_card, keyword_count,
};
pub use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
pub use crate::common_k702_111_124::{
    assert_supported_card, castable, gain, graveyard, has, on_stack, pregame, run, tokens_of,
    untapped_lands,
};
use mtg_engine::card::card;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts `n` copies of a real card into `p`'s graveyard.
pub fn graveyard_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// Puts `n` copies of a real card onto the battlefield under `p`'s control.
pub fn battlefield_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, name)).collect()
}

/// The methods `p` could cast `card` with right now.
pub fn cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.cast_options(p, card)
        .into_iter()
        .filter(|o| t.g.can_begin_cast(p, card, o))
        .map(|o| o.method)
        .collect()
}

/// Whether the legal actions of `p` include casting `card` (any way).
pub fn can_cast_now(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// Whether the legal actions of `p` include activating an ability of `source`.
pub fn can_activate_now(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source: s, .. } if *s == source))
}

/// Permanents `p` controls named `name`.
pub fn named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// A creature token's colors, types, and P/T summary.
pub fn is_token(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).kind == mtg_engine::object::ObjKind::Token
}

/// The number of +1/+1 counters on an object (followed across zone changes).
pub fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// The keyword kind's name, for [`triggers_on_stack`].
pub fn kw_name(k: KeywordKind) -> &'static str {
    k.name()
}

/// A real card's definition.
pub fn def(name: &str) -> mtg_engine::card::CardDef {
    card(name).as_ref().clone()
}

/// Whether the card is in exile (followed across zone changes).
pub fn exiled(t: &TestGame, id: ObjectId) -> bool {
    matches!(t.zone(id), Zone::Exile)
}
