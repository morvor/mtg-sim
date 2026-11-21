//! Shared helpers for the CR 601–602/118 audit tests (`r601_audit_*.rs`): card support
//! checks, mana pool and tapped-land queries, and shortcuts for what a player was asked
//! and offered while casting.

#![allow(dead_code)]

use mtg_engine::card::card;
use mtg_engine::casting::CastOption;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts the compiler fully supports `name` (a test on a half-supported card would
/// test the wrong thing).
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

pub fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

pub fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.player(p).mana_pool.total()
}

pub fn pool_count(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.player(p).mana_pool.count(ty)
}

/// The untapped lands `p` controls.
pub fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.is(CardType::Land) && !o.tapped
        })
        .count()
}

/// The tapped lands `p` controls.
pub fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.is(CardType::Land) && o.tapped
        })
        .count()
}

pub fn is_tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Whether casting `card` with `method` is among `p`'s legal actions now.
pub fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Cast { card, method })
}

/// The ways `p` could cast `card` right now.
pub fn options(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
    t.g.recompute();
    t.g.cast_options(p, card)
}

/// The X recorded for a spell on the stack.
pub fn x_of(t: &TestGame, spell: ObjectId) -> Option<i32> {
    t.g.obj(spell).stack.as_ref().and_then(|s| s.x)
}

/// The mana spent to cast `spell` (how many mana, from its cast information).
pub fn mana_spent(t: &TestGame, spell: ObjectId) -> usize {
    t.g.obj(t.g.current(spell))
        .stack
        .as_ref()
        .map_or(0, |s| s.cast.mana_spent.len())
}

/// The greatest X `p` was last offered, if X was asked.
pub fn offered_x_max(t: &TestGame, p: PlayerId) -> Option<i64> {
    t.asked().iter().rev().find_map(|(q, d)| match d {
        Decision::ChooseX { max, .. } if *q == p => Some(*max),
        _ => None,
    })
}

/// How many times `p` was asked to choose X.
pub fn x_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::ChooseX { .. }))
        .count()
}

/// The names of the optional costs `p` was asked about.
pub fn optional_costs_asked(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.asked()
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::OptionalCost { name, .. } if *q == p => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// The kinds of the decisions `p` was asked, in order, as short labels.
pub fn decision_labels(t: &TestGame, p: PlayerId) -> Vec<&'static str> {
    t.asked()
        .iter()
        .filter(|(q, _)| *q == p)
        .map(|(_, d)| match d {
            Decision::ChooseX { .. } => "x",
            Decision::ChooseTargets { .. } => "targets",
            Decision::ChooseModes { .. } => "modes",
            Decision::OptionalCost { .. } => "optional cost",
            Decision::ChooseEntities { .. } => "entities",
            Decision::ChooseOption { .. } => "option",
            Decision::ChooseCastingMethod { .. } => "method",
            Decision::YesNo { .. } => "yes/no",
            Decision::Priority { .. } => "priority",
            _ => "other",
        })
        .collect()
}

/// Queues the creatures (or cards) to tap, exile or sacrifice for convoke, delve,
/// improvise or emerge: an entities choice.
pub fn choose_objects(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) {
    let v: Vec<Entity> = ids.iter().map(|c| Entity::Object(*c)).collect();
    t.answer(p, DecisionKind::Entities, Answer::Entities(v));
}

/// Queues the answer to a "choose an option" question (a way of casting, a half, how to
/// pay a symbol).
pub fn choose_option(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// Taps `land` for mana with priority (its first activated ability).
pub fn tap_for_mana(t: &mut TestGame, p: PlayerId, land: ObjectId) {
    t.activate(p, land, 0, &[]).expect("a mana ability");
}

pub fn creatures_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name).len()
}

/// The permanents on the battlefield with the subtype `s` (tokens included).
pub fn permanents_with_subtype(t: &TestGame, s: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype(s))
        .count()
}
