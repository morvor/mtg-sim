//! Shared helpers for the tests of rulings batch P071 (`r_p071_*.rs`): cards that let a
//! player cast or play cards from exile or from a graveyard, and cards that give
//! deathtouch or double strike.

#![allow(dead_code)]

pub use crate::r_p069_common::{
    cast_free, castable, exiled, land_timing, mana, must_pay, no_mana, o, playable_land,
    sorcery_timing_only,
};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, stack_library, supported, with_subtype,
};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's creatures `attackers` attack P1 unblocked and deal combat damage; the game runs to
/// the end of combat step (with the triggers resolved).
pub fn connect(t: &mut TestGame, attackers: &[ObjectId]) {
    let a: Vec<_> = attackers.iter().map(|c| (*c, Entity::Player(P1))).collect();
    attack_with(t, &a);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
}

/// P1 casts Lightning Bolt at themselves (a spell on the stack); P1 holds priority.
pub fn p1_bolt_on_stack(t: &mut TestGame) -> ObjectId {
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(t, P1, ManaType::R, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P1).go()
}

/// Asserts, without leaving the current turn of `p` (a permission lasting "this turn"),
/// that `p` can cast `card` (paying `n` mana of type `ty`) only in a main phase with an
/// empty stack: not in combat, not with a spell on the stack. Ends in `p`'s postcombat
/// main phase with the mana in the pool.
pub fn sorcery_timing_this_turn(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    ty: ManaType,
    n: u32,
) {
    no_mana(t, p);
    mana(t, p, ty, n);
    t.set_step(p, Step::DeclareAttackers);
    assert!(!castable(t, p, card), "combat: {}", t.dump_log());
    t.set_step(p, Step::PostcombatMain);
    assert!(castable(t, p, card), "main phase, empty stack: {}", t.dump_log());
    let bolt = p1_bolt_on_stack(t);
    assert!(!castable(t, p, card), "stack not empty");
    t.resolve_all();
    assert_eq!(t.zone(bolt), mtg_engine::object::Zone::Graveyard(P1));
    t.set_step(p, Step::PostcombatMain);
    no_mana(t, p);
    mana(t, p, ty, n);
    assert!(castable(t, p, card));
}

/// Asserts, without leaving the current turn of `p`, that `p` can play the land `card`
/// only in a main phase with an empty stack and a land play left; ends with the land
/// play used on a Wastes.
pub fn land_timing_this_turn(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    t.set_step(p, Step::DeclareAttackers);
    assert!(!playable_land(t, p, card), "combat");
    t.set_step(p, Step::PostcombatMain);
    assert!(playable_land(t, p, card), "{}", t.dump_log());
    p1_bolt_on_stack(t);
    assert!(!playable_land(t, p, card), "stack not empty");
    t.resolve_all();
    t.set_step(p, Step::PostcombatMain);
    assert!(playable_land(t, p, card));
    let w = t.hand(p, "Wastes");
    t.play_land(p, w).unwrap();
    assert!(!playable_land(t, p, card), "land play used");
}
