//! Volcano Hellion (hand-written, `src/cards/volcano_hellion.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn deals_the_chosen_amount_to_you_and_the_target() {
    cr!("603.2");
    ruling!("Volcano Hellion", "you choose a number");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.enter(P0, "Volcano Hellion");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn zero_may_be_chosen() {
    cr!("603.2");
    ruling!("Volcano Hellion", "You may choose 0");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    t.enter(P0, "Volcano Hellion");
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn the_damage_cant_be_prevented() {
    cr!("615.12");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    // "Prevent all damage that would be dealt this turn."
    t.lands(P1, "Plains", 1);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Number, Answer::Number(4));
    let fog = t.hand(P0, "Holy Day");
    t.lands(P0, "Plains", 1);
    t.cast(P0, fog).go();
    t.resolve();
    t.enter(P0, "Volcano Hellion");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 16);
}
