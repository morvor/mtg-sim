//! Starlight Spectacular (hand-written, `src/cards/starlight_spectacular.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn each_creature_gets_a_bonus_for_those_chosen_before_it() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlight Spectacular");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Gray Ogre");
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 0, 1]));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // Order: Gray Ogre (+0), Grizzly Bears (+1), Hill Giant (+2).
    assert_eq!(t.pt(c), (2, 2));
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (5, 5));
}
