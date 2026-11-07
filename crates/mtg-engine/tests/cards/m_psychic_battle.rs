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

#[test]
fn a_player_with_an_empty_library_has_no_card_to_compare() {
    cr!("701.20a");
    ruling!(
        "Psychic Battle",
        "An empty library will have no card to reveal, so a player with an empty library has no card to compare. The effect looks for the highest cost card of the cards that actually are revealed."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Battle");
    t.g.players[0].library.clear();
    // P1's land (mana value 0) is the only card revealed, so it's the greatest.
    t.library_top(P1, "Forest");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 20));
}

#[test]
fn with_no_card_revealed_the_targets_stay() {
    cr!("701.20a");
    ruling!(
        "Psychic Battle",
        "An empty library will have no card to reveal, so a player with an empty library has no card to compare."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Battle");
    t.g.players[0].library.clear();
    t.g.players[1].library.clear();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    // Nobody may change the target.
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
}

#[test]
fn a_split_cards_mana_value_is_that_of_both_halves() {
    cr!("709.4b");
    ruling!(
        "Psychic Battle",
        "The mana value of a split card is determined by the combined mana cost of its two halves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Battle");
    // Fire // Ice ({1}{R} and {1}{U}: 4) beats Divination (3); either half alone wouldn't.
    t.library_top(P0, "Divination");
    t.library_top(P1, "Fire // Ice");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 20));
}
