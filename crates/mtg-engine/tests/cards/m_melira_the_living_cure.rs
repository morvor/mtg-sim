//! Melira, the Living Cure (hand-written, `src/cards/melira_the_living_cure.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn one_poison_counter_then_no_more_this_turn() {
    cr!("614.1a");
    ruling!("Melira, the Living Cure", "additional events that would put poison counters on a player don't happen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melira, the Living Cure");
    t.g.add_counters(Entity::Player(P0), "poison", 3, None);
    assert_eq!(t.g.player(P0).poison(), 1);
    t.g.add_counters(Entity::Player(P0), "poison", 2, None);
    assert_eq!(t.g.player(P0).poison(), 1);
    // Opponents aren't affected.
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    assert_eq!(t.g.player(P1).poison(), 3);
}
