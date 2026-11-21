//! Kaya, Ghost Assassin (hand-written, `src/cards/kaya_ghost_assassin.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn exiles_the_target_and_returns_it_next_upkeep() {
    cr!("606.3", "603.7");
    ruling!("Kaya, Ghost Assassin", "you choose whether to exile Kaya or that creature as the ability resolves");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya, Ghost Assassin");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, kaya, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(kaya));
    assert_eq!(t.life(P0), 18);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P1);
}

#[test]
fn may_exile_kaya_instead() {
    cr!("606.3");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya, Ghost Assassin");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, kaya, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Kaya, Ghost Assassin"));
    assert!(t.on_battlefield(bear));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kaya, Ghost Assassin").len(), 1);
}

#[test]
fn without_a_target_kaya_is_exiled() {
    cr!("606.3");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya, Ghost Assassin");
    t.activate(P0, kaya, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Kaya, Ghost Assassin"));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn an_illegal_target_means_nothing_happens() {
    cr!("608.2b");
    ruling!("Kaya, Ghost Assassin", "You won’t exile Kaya or lose 2 life");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya, Ghost Assassin");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, kaya, 0, &[Entity::Object(bear)]).unwrap();
    t.g.destroy(bear, None);
    t.resolve_all();
    assert!(t.on_battlefield(kaya));
    assert_eq!(t.life(P0), 20);
}
