//! Sylvan Library (hand-written, `src/cards/sylvan_library.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn pay_4_life_or_put_back_two_cards_drawn_this_turn() {
    cr!("504.1", "603.2");
    ruling!("Sylvan Library", "You always perform your normal draw before this ability");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sylvan Library");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    // Keep one (pay 4 life), put the other back.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    // Drew 1 + 2, chose two of them: one paid for, one put back.
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn declining_draws_nothing_extra() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sylvan Library");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 20);
}
