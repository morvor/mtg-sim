//! Rulings batch P137 — the other cards whose wording the batch's compiler and engine
//! changes reach: "each opponent who can't" after an edict (Skull Storm, Momentum
//! Breaker) and a modal spell's paragraph after its modes (Thermal Flux).

use crate::r_p137_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Supertype;
use mtg_engine::*;

#[test]
fn skull_storm_opponents_who_sacrificed_nothing_lose_half_their_life() {
    cr!("701.21a", "107.1a");
    supported("Skull Storm");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.players[P2.idx()].life = 7;
    give_mana_for(&mut t, P0, "Skull Storm");
    let spell = t.hand(P0, "Skull Storm");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // P1 sacrificed a creature: no life loss. P2 couldn't: loses half of 7, rounded up.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 3);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn momentum_breaker_opponents_who_sacrificed_nothing_discard() {
    cr!("701.21a", "701.9a");
    supported("Momentum Breaker");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    // A Vehicle that isn't a creature can still be the sacrificed permanent.
    let copter = t.battlefield(P1, "Smuggler's Copter");
    t.hand(P1, "Island");
    t.hand(P2, "Island");
    t.enter(P0, "Momentum Breaker");
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(copter));
    assert_eq!(t.hand_size(P1), 1, "P1 sacrificed its Vehicle");
    assert_eq!(t.hand_size(P2), 0, "P2 had nothing to sacrifice");
}

#[test]
fn thermal_flux_draws_at_the_next_upkeep_after_either_mode() {
    cr!("608.2c", "700.2");
    supported("Thermal Flux");
    for mode in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let land = if mode == 0 {
            t.battlefield(P1, "Island")
        } else {
            t.battlefield(P1, "Snow-Covered Island")
        };
        for _ in 0..3 {
            t.library_top(P0, "Island");
        }
        give_mana_for(&mut t, P0, "Thermal Flux");
        let spell = t.hand(P0, "Thermal Flux");
        t.cast(P0, spell).modes(&[mode]).targets(&[obj(land)]).go();
        t.resolve_all();
        let snow = t.obj(land).chars.has_supertype(Supertype::Snow);
        assert_eq!(snow, mode == 0);
        let hand = t.hand_size(P0);
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "one delayed draw, mode {mode}");
        t.advance_to(P1, Step::Draw);
        assert_eq!(t.hand_size(P0), hand + 1, "drawn only once, mode {mode}");
    }
}

#[test]
fn single_combat_restriction_lasts_until_the_end_of_your_next_turn() {
    cr!("611.2a", "514.2");
    supported("Single Combat");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    give_mana_for(&mut t, P0, "Single Combat");
    let spell = t.hand(P0, "Single Combat");
    t.cast(P0, spell).go();
    t.resolve_all();
    let castable = |t: &mut TestGame, p: PlayerId, name: &str| {
        let card = t.hand(p, name);
        give_mana_for(t, p, name);
        let ok = t.cast(p, card).try_go().is_ok();
        t.g.players[p.idx()].mana_pool.empty();
        ok
    };
    // The opponent's turn: no creature spells, but other spells.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P1, "Grizzly Bears"));
    assert!(castable(&mut t, P1, "Divination"));
    t.resolve_all();
    // Your next turn: still no creature spells, for anyone.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // After the end of your next turn, the restriction is gone.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(castable(&mut t, P1, "Grizzly Bears"));
}
