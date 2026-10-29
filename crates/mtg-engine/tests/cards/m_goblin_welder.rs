//! Goblin Welder (hand-written, `src/cards/goblin_welder.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn swaps_an_artifact_and_an_artifact_card_from_that_players_graveyard() {
    cr!("115.1", "608.2b");
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Goblin Welder");
    let ring = t.battlefield(P1, "Sol Ring");
    let stone = t.graveyard(P1, "Mind Stone");
    t.activate(P0, welder, 0, &[Entity::Object(ring), Entity::Object(stone)])
        .unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
    let s = t.named_on_battlefield("Mind Stone");
    assert_eq!(s.len(), 1);
    assert_eq!(t.g.obj(s[0]).controller, P1);
}

#[test]
fn nothing_happens_if_a_target_became_illegal() {
    cr!("608.2b");
    ruling!("Goblin Welder", "nothing gets sacrificed and nothing gets returned");
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Goblin Welder");
    let ring = t.battlefield(P1, "Sol Ring");
    let stone = t.graveyard(P1, "Mind Stone");
    t.activate(P0, welder, 0, &[Entity::Object(ring), Entity::Object(stone)])
        .unwrap();
    // The card leaves the graveyard first.
    t.g.exile_object(stone, None);
    t.resolve_all();
    assert!(t.on_battlefield(ring));
}

#[test]
fn the_card_must_be_in_that_players_graveyard() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Goblin Welder");
    let ring = t.battlefield(P1, "Sol Ring");
    let stone = t.graveyard(P0, "Mind Stone");
    let r = t.activate(P0, welder, 0, &[Entity::Object(ring), Entity::Object(stone)]);
    t.resolve_all();
    // Either the activation was refused or nothing was swapped: the targets aren't
    // related.
    assert!(r.is_err() || t.on_battlefield(ring));
    assert!(t.in_graveyard(P0, "Mind Stone"));
}
