//! Shared helpers for the tests of rulings batch S27 (`r_s27_*.rs`): costs — the value
//! of {X} in copied and referenced mana costs (CR 107.3), activated abilities and their
//! costs (CR 602, 605, 107.6), cost reductions and alternative and additional costs
//! (CR 118, 601.2f), and the mana spent on a spell. (The helpers of batches S01–S25 are
//! used too.)

#![allow(dead_code)]

use crate::r_s08_common::mana_value;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Ingenious Prodigy ({X}{U} 0/1 skulk, "This creature enters with X +1/+1 counters on
/// it."), cast by `p` with X = 3: a 3/4 whose mana value on the battlefield is 1.
pub fn prodigy_x3(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.lands(p, "Island", 1);
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, "Ingenious Prodigy");
    t.cast(p, card).x(3).go();
    t.resolve_all();
    let id = t.g.current(card);
    assert!(t.on_battlefield(id));
    assert_eq!(t.counters(id, "+1/+1"), 3);
    assert_eq!(t.pt(id), (3, 4));
    assert_eq!(mana_value(t, id), 1);
    id
}

/// `copy` is an Ingenious Prodigy for which X was 0: it entered with no +1/+1 counters
/// (a 0/1), and its mana value is 1 (its {X}{U} with X = 0).
pub fn prodigy_copy_has_x_0(t: &TestGame, copy: ObjectId) {
    assert!(t.on_battlefield(copy));
    assert_eq!(t.obj_now(copy).chars.name, "Ingenious Prodigy");
    assert_eq!(t.counters(copy, "+1/+1"), 0);
    assert_eq!(t.pt(copy), (0, 1));
    assert_eq!(mana_value(t, copy), 1);
}

/// Chalice of the Void ({X}{X}, "This artifact enters with X charge counters on it."),
/// cast by `p` with X = 2.
pub fn chalice_x2(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.lands(p, "Wastes", 4);
    let card = t.hand(p, "Chalice of the Void");
    t.cast(p, card).x(2).go();
    t.resolve_all();
    let id = t.g.current(card);
    assert!(t.on_battlefield(id));
    assert_eq!(t.counters(id, "charge"), 2);
    assert_eq!(mana_value(t, id), 0);
    id
}

/// `copy` is a Chalice of the Void for which X was 0: no charge counters, mana value 0.
pub fn chalice_copy_has_x_0(t: &TestGame, copy: ObjectId) {
    assert!(t.on_battlefield(copy));
    assert_eq!(t.obj_now(copy).chars.name, "Chalice of the Void");
    assert_eq!(t.counters(copy, "charge"), 0);
    assert_eq!(mana_value(t, copy), 0);
}

/// Mana Bloom ({X}{G}, "This enchantment enters with X charge counters on it."), cast by
/// `p` with X = 3. Returns the card (the spell is left to resolve).
pub fn cast_mana_bloom_x3(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.lands(p, "Forest", 1);
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, "Mana Bloom");
    t.cast(p, card).x(3).go();
    card
}

/// `copy` is a Mana Bloom for which X was 0: no charge counters, mana value 1.
pub fn bloom_copy_has_x_0(t: &TestGame, copy: ObjectId) {
    assert!(t.on_battlefield(copy));
    assert_eq!(t.obj_now(copy).chars.name, "Mana Bloom");
    assert_eq!(t.counters(copy, "charge"), 0);
    assert_eq!(mana_value(t, copy), 1);
}

/// The tokens named `name` on the battlefield.
pub fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj(*id).is_token())
        .collect()
}

/// The texts of the activated abilities (mana abilities included) `p` could activate of
/// `source` now.
pub fn activatable(t: &mut TestGame, p: PlayerId, source: ObjectId) -> Vec<String> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let s = t.g.current(source);
    t.g.activatable_abilities(p)
        .into_iter()
        .filter(|(id, _)| *id == s)
        .map(|(_, a)| a.text.to_string())
        .collect()
}

/// Whether `p` could activate an ability of `source` whose text contains `needle` now.
pub fn can_activate_containing(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
) -> bool {
    activatable(t, p, source).iter().any(|a| a.contains(needle))
}

/// The uid of the activated ability of `source` whose text contains `needle`.
pub fn ability_containing(t: &mut TestGame, source: ObjectId, needle: &str) -> u64 {
    t.g.recompute();
    let s = t.g.current(source);
    t.g.obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no activated ability containing {needle:?}"))
}
