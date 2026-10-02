//! Arboria (hand-written, `src/cards/arboria.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_player_who_did_nothing_last_turn_cant_be_attacked() {
    cr!("508.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arboria");
    let b = t.battlefield(P1, "Grizzly Bears");
    let a = t.battlefield(P0, "Grizzly Bears");
    // P0's turn 2: P0 does nothing.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    let _ = a;
}

#[test]
fn a_player_who_cast_a_spell_last_turn_can_be_attacked() {
    cr!("508.1c");
    ruling!("Arboria", "If a player cast a spell during their last turn but that spell was countered");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arboria");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn playing_a_land_counts() {
    cr!("508.1c", "305.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arboria");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).unwrap();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
}
