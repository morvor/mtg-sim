//! The Face of Boe (hand-written, `src/cards/the_face_of_boe.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn casts_a_suspend_card_for_its_suspend_cost() {
    cr!("118.9", "702.62a");
    ruling!("The Face of Boe", "You cast the spell immediately as The Face of Boe's ability resolves");
    let mut t = TestGame::new(2);
    let boe = t.battlefield(P0, "The Face of Boe");
    // Ancestral Vision: no mana cost; suspend 4—{U}. "Target player draws three cards."
    let av = t.hand(P0, "Ancestral Vision");
    t.lands(P0, "Island", 1);
    t.answer_choose(P0, &[Entity::Object(av)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.activate(P0, boe, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
}
