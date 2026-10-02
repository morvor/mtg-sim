//! Peace Talks (hand-written, `src/cards/peace_talks.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn peace(t: &mut TestGame) {
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Peace Talks");
    t.cast(P0, s).go();
    t.resolve();
}

#[test]
fn no_targeting_players_or_permanents_this_turn() {
    cr!("115.4", "601.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    peace(&mut t);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(bears).try_go().is_err());
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn no_attacks_this_turn_and_next_turn_then_normal() {
    cr!("508.1c");
    ruling!("Peace Talks", "It affects the current turn and the next turn.");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    peace(&mut t);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    // The turn after next is unaffected.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}
