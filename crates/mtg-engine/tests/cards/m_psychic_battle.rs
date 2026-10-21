//! Psychic Battle (hand-written, `src/cards/psychic_battle.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_highest_reveal_may_change_the_targets() {
    cr!("115.7", "701.20a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Battle");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    // P1 revealed the greater mana value: they redirect the Bolt to P0.
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_tie_leaves_the_targets() {
    cr!("115.7");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Battle");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
