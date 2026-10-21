//! Shaman's Trance (hand-written, `src/cards/shamans_trance.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn you_may_flash_back_an_opponents_card() {
    cr!("609.4", "702.34a");
    ruling!("Shaman's Trance", "You can use Flashback on cards to cast them.");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let trance = t.hand(P0, "Shaman's Trance");
    t.cast(P0, trance).go();
    t.resolve_all();
    // Firebolt: flashback {4}{R}; P0 has five more Mountains.
    t.lands(P0, "Mountain", 5);
    let firebolt = t.graveyard(P1, "Firebolt");
    let options = t.g.cast_options(P0, firebolt);
    assert!(!options.is_empty());
    t.cast(P0, firebolt).method(options[0].method.clone()).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Firebolt"));
}

#[test]
fn other_players_cant_cast_spells_from_their_graveyards() {
    cr!("609.4");
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 3);
    let think = t.graveyard(P1, "Think Twice");
    let castable = |t: &TestGame| {
        t.g.cast_options(P1, think)
            .iter()
            .any(|o| t.g.can_begin_cast(P1, think, o))
    };
    t.g.turn.priority = Some(P1);
    assert!(castable(&t));
    t.lands(P0, "Mountain", 3);
    let trance = t.hand(P0, "Shaman's Trance");
    t.cast(P0, trance).go();
    t.resolve_all();
    t.g.turn.priority = Some(P1);
    assert!(!castable(&t));
}
