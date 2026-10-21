//! Shrouded Lore (hand-written, `src/cards/shrouded_lore.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn you_get_the_last_card_chosen() {
    cr!("608.2c");
    ruling!("Shrouded Lore", "As soon as you decline to pay {B}, the last card that was chosen");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Shrouded Lore");
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_yes(P0, true);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.answer_yes(P0, false);
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn without_paying_you_get_the_first() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 1);
    let s = t.hand(P0, "Shrouded Lore");
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
