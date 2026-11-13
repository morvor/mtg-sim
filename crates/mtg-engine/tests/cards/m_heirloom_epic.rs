//! Heirloom Epic (hand-written, `src/cards/heirloom_epic.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn creatures_may_be_tapped_rather_than_paying_mana() {
    cr!("602.2b", "601.2h");
    ruling!("Heirloom Epic", "even one you haven't controlled continuously since the beginning of your most recent turn");
    let mut t = TestGame::new(2);
    let epic = t.battlefield(P0, "Heirloom Epic");
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield_sick(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Forest", 1);
    let es: Vec<Entity> = bears.iter().map(|b| Entity::Object(*b)).collect();
    t.answer_choose(P0, &es);
    t.activate(P0, epic, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    for b in bears {
        assert!(t.obj_now(b).tapped);
    }
}

#[test]
fn not_without_enough_mana_and_creatures() {
    cr!("602.2b");
    let mut t = TestGame::new(2);
    let epic = t.battlefield(P0, "Heirloom Epic");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(t.activate(P0, epic, 0, &[]).is_err());
}
