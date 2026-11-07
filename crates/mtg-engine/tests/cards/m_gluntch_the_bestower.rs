//! Gluntch, the Bestower (hand-written, `src/cards/gluntch_the_bestower.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn three_different_players_get_the_three_gifts() {
    cr!("603.2");
    ruling!("Gluntch, the Bestower", "All three players must be different players");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gluntch, the Bestower");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    assert_eq!(t.hand_size(P2), 1);
    let treasures = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|&o| t.g.obj(o).chars.subtypes.iter().any(|s| s == "Treasure"))
        .count();
    assert_eq!(treasures, 2);
}

#[test]
fn with_two_players_no_one_gets_treasures() {
    cr!("603.2");
    ruling!("Gluntch, the Bestower", "no one will create Treasure tokens");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gluntch, the Bestower");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    let treasures = t
        .g
        .permanents()
        .filter(|o| o.chars.subtypes.iter().any(|s| s == "Treasure"))
        .count();
    assert_eq!(treasures, 0);
}
