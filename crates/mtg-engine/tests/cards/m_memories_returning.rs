//! Memories Returning (hand-written, `src/cards/memories_returning.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn you_and_an_opponent_alternate_picks() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    let names = ["Island", "Forest", "Swamp", "Mountain", "Plains"];
    let cards: Vec<ObjectId> = names.iter().map(|n| t.library_top(P0, n)).collect();
    // Top first: Plains, Mountain, Swamp, Forest, Island.
    t.answer_choose(P0, &[Entity::Object(cards[4])]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(cards[3])]);
    t.answer_choose(P0, &[Entity::Object(cards[2])]);
    t.answer_choose(P1, &[Entity::Object(cards[1])]);
    t.lands(P0, "Island", 4);
    let s = t.hand(P0, "Memories Returning");
    t.cast(P0, s).go();
    t.resolve();
    for n in ["Plains", "Swamp", "Island"] {
        assert!(t.in_hand(P0, n), "{n}");
    }
    let lib = &t.g.player(P0).library;
    // The two the opponent chose are on the bottom.
    assert_eq!(lib[0], cards[1]);
    assert_eq!(lib[1], cards[3]);
}
