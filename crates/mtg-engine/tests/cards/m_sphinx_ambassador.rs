//! Sphinx Ambassador (hand-written, `src/cards/sphinx_ambassador.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn takes_the_creature_if_the_name_is_wrong() {
    cr!("701.23a", "510.2");
    ruling!("Sphinx Ambassador", "If it doesn’t, but it’s a creature card, you may put it onto the battlefield");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx Ambassador");
    let wurm = t.library_top(P1, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer(P1, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_yes(P0, true);
    t.attack(&[(sphinx, Entity::Player(P1))], &[]);
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert_eq!(t.g.obj(w[0]).controller, P0);
}

#[test]
fn naming_it_stops_you() {
    cr!("701.23a");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx Ambassador");
    let wurm = t.library_top(P1, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer(P1, DecisionKind::Name, Answer::Text("Craw Wurm".into()));
    t.answer_yes(P0, true);
    t.attack(&[(sphinx, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    assert_eq!(t.library_size(P1), 31);
}
