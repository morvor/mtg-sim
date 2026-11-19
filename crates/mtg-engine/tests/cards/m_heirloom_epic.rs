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

#[test]
fn by_default_only_the_creatures_the_mana_cant_pay_for_are_tapped() {
    cr!("602.2b", "601.2h");
    interp!("default-cost-helpers");
    // No answer queued: with two Wastes, two of the three Grizzly Bears are tapped; with
    // four Wastes, none.
    for (wastes, n) in [(2, 2), (4, 0)] {
        let mut t = TestGame::new(2);
        let epic = t.battlefield(P0, "Heirloom Epic");
        let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
        let lands = t.lands(P0, "Wastes", wastes);
        t.activate(P0, epic, 0, &[]).unwrap();
        assert_eq!(bears.iter().filter(|b| t.obj_now(**b).tapped).count(), n);
        assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 1);
    }
}
