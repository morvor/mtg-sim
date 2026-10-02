//! Mana Maze (hand-written, `src/cards/mana_maze.rs`): players can't cast spells that
//! share a color with the spell most recently cast this turn.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn same_color_as_the_last_spell_cant_be_cast() {
    cr!("601.3");
    ruling!("Mana Maze", "Does not affect the first spell cast each turn.");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mana Maze");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 2);
    let b1 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(P1).go();
    t.resolve();
    let b2 = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, b2).target(P1).try_go().is_err());
    // A green spell doesn't share a color with it.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Now red is fine again.
    let b2 = t.g.current(b2);
    t.cast(P0, b2).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn the_restriction_ends_with_the_turn() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mana Maze");
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let b1 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(P1).go();
    t.resolve();
    t.advance_to(P1, Step::PrecombatMain);
    let b2 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b2).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
}
