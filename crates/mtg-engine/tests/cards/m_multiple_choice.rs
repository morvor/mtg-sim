//! Multiple Choice (hand-written, `src/cards/multiple_choice.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn x_four_or_more_does_all_of_the_above() {
    cr!("107.3");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let mc = t.hand(P0, "Multiple Choice");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.cast(P0, mc).x(4).go();
    t.resolve_all();
    // Scry 1 then draw; P1 returned a creature; a 4/4 Elemental.
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    let _ = bear;
    let tokens: Vec<_> = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|&o| t.g.obj(o).chars.subtypes.iter().any(|s| s == "Elemental"))
        .collect();
    assert_eq!(tokens.len(), 1);
}

#[test]
fn x_three_only_makes_the_token() {
    cr!("107.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let mc = t.hand(P0, "Multiple Choice");
    t.cast(P0, mc).x(3).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    let tokens = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|&o| t.g.obj(o).chars.subtypes.iter().any(|s| s == "Elemental"))
        .count();
    assert_eq!(tokens, 1);
}
