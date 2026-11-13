//! Shared helpers for the tests of rulings batch P069 (`r_p069_*.rs`): cards that let a
//! player cast (or play) cards from exile.

#![allow(dead_code)]

pub use crate::r_p067_common::{castable, exiled, mana, o, playable_land};
use mtg_engine::decision::Action;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` leaves the game by conceding (CR 104.3a, 800.4a).
pub fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
    t.settle();
}

/// Casts `card` (followed across zone changes) for `p` normally, paying with the mana in
/// the pool.
pub fn cast_now(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> ObjectId {
    let c = t.g.current(card);
    t.cast(p, c).targets(targets).go()
}

/// Casts `card` (followed across zone changes) for `p` without paying its mana cost.
pub fn cast_free(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> ObjectId {
    let c = t.g.current(card);
    t.cast(p, c)
        .method(CastMethod::Free)
        .targets(targets)
        .go()
}

/// Asserts that `card`, owned by P1, is (as the object it is now) a permanent P0 controls,
/// then that P0 leaving the game exiles it (CR 800.4a).
pub fn p0_leaves_and_its_permanent_is_exiled(t: &mut TestGame, card: ObjectId) {
    assert!(t.on_battlefield(card), "{}", t.dump_log());
    assert_eq!(t.obj_now(card).controller, P0);
    assert_eq!(t.obj_now(card).owner, P1);
    concede(t, P0);
    assert!(!t.g.player(P0).in_game());
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.obj_now(card).owner, P1);
}

/// Asserts that `spell`, owned by P1, is a spell P0 controls on the stack, then that P0
/// leaving the game exiles it (CR 800.4a): it never resolves.
pub fn p0_leaves_and_its_spell_is_exiled(t: &mut TestGame, spell: ObjectId) {
    assert_eq!(t.zone(spell), Zone::Stack, "{}", t.dump_log());
    assert_eq!(t.obj_now(spell).controller, P0);
    assert_eq!(t.obj_now(spell).owner, P1);
    concede(t, P0);
    assert_eq!(t.zone(spell), Zone::Exile);
    assert!(t.g.stack.is_empty());
}

/// Asserts that P1 leaving the game takes `card` (a card P1 owns, wherever it is) with
/// them: it is no longer in any zone of the game (CR 800.4a).
pub fn p1_leaves_and_takes_the_card(t: &mut TestGame, card: ObjectId) {
    let name = t.obj_now(card).chars.name.to_string();
    concede(t, P1);
    assert!(!t.g.player(P1).in_game());
    assert!(t.named_on_battlefield(&name).is_empty(), "{name}");
    assert!(!t.in_exile(&name), "{name}");
    assert!(t.g.stack.is_empty());
}

/// Jumps to P0's precombat main phase with priority.
pub fn main0(t: &mut TestGame) {
    t.set_step(P0, Step::PrecombatMain);
}

/// Empties `p`'s mana pool.
pub fn no_mana(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].mana_pool.empty();
}

/// Asserts that `p` (in a two-player game) can cast `card` in its main phase with an
/// empty stack only (not with a spell on the stack, not in combat, not in the other
/// player's turn), paying `n` mana of type `ty` (0: free).
pub fn sorcery_timing_only(t: &mut TestGame, p: PlayerId, card: ObjectId, ty: ManaType, n: u32) {
    let other = PlayerId(1 - p.0);
    t.set_step(p, Step::PostcombatMain);
    no_mana(t, p);
    mana(t, p, ty, n);
    assert!(castable(t, p, card), "main phase, empty stack: {}", t.dump_log());
    let bolt = t.hand(other, "Lightning Bolt");
    mana(t, other, ManaType::R, 1);
    t.g.turn.priority = Some(other);
    t.cast(other, bolt).target(other).go();
    assert!(!castable(t, p, card), "stack not empty");
    t.resolve_all();
    t.set_step(p, Step::BeginningOfCombat);
    assert!(!castable(t, p, card), "combat");
    t.set_step(other, Step::PrecombatMain);
    assert!(!castable(t, p, card), "another player's turn");
    t.set_step(p, Step::PrecombatMain);
    no_mana(t, p);
    mana(t, p, ty, n);
    assert!(castable(t, p, card));
}

/// Asserts that `p` can't cast `card` now without mana but can with `n` mana of type `ty`
/// (the card's mana cost has to be paid).
pub fn must_pay(t: &mut TestGame, p: PlayerId, card: ObjectId, ty: ManaType, n: u32) {
    no_mana(t, p);
    assert!(!castable(t, p, card), "cast without paying");
    mana(t, p, ty, n);
    assert!(castable(t, p, card), "{}", t.dump_log());
    no_mana(t, p);
}

/// Asserts that `p` can play the land `card` only in its main phase with an empty stack
/// and a land play left (CR 305.2, 305.3); ends in `p`'s main phase with the land play
/// used on another land.
pub fn land_timing(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    let other = PlayerId(1 - p.0);
    t.set_step(p, Step::BeginningOfCombat);
    assert!(!playable_land(t, p, card), "combat");
    t.set_step(other, Step::PrecombatMain);
    assert!(!playable_land(t, p, card), "another player's turn");
    t.set_step(p, Step::PrecombatMain);
    let bolt = t.hand(other, "Lightning Bolt");
    mana(t, other, ManaType::R, 1);
    t.g.turn.priority = Some(other);
    t.cast(other, bolt).target(other).go();
    assert!(!playable_land(t, p, card), "stack not empty");
    t.resolve_all();
    t.set_step(p, Step::PrecombatMain);
    assert!(playable_land(t, p, card), "{}", t.dump_log());
    let w = t.hand(p, "Wastes");
    t.play_land(p, w).unwrap();
    assert!(!playable_land(t, p, card), "land play used");
}
