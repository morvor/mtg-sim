//! Phyrexian Splicer (hand-written, `src/cards/phyrexian_splicer.rs`).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn moves_the_chosen_keyword() {
    cr!("602.2b", "613.1f");
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Phyrexian Splicer");
    let angel = t.battlefield(P1, "Serra Angel");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, sp, 0, &[Entity::Object(angel), Entity::Object(bear)])
        .unwrap();
    t.resolve_all();
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
    assert!(t.obj_now(bear).has_keyword(KeywordKind::Flying));
}
