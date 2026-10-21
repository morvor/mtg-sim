//! The worked examples of CR 120 (damage), played out as tests.

use super::r105_util::{put, vanilla};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn prevention_that_gains_life_is_applied_before_a_life_floor_looks_at_the_event() {
    cr!("120.4", "120.4b", "120.4d");
    cr_example!("120.4d", "Worship's effect is not applied");
    let mut t = TestGame::new(2);
    // The defending player controls a creature and Worship ("If you control a creature,
    // damage that would reduce your life total to less than 1 reduces it to 1
    // instead."), and is at 2 life.
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Worship");
    t.g.players[P1.idx()].life = 2;
    let a = put(&mut t, P0, vanilla("Big One", "{4}{G}", 5, 5));
    let b = put(&mut t, P0, vanilla("Big Two", "{4}{G}", 5, 5));
    // Awe Strike: "The next time target creature would deal damage this turn, prevent
    // that damage. You gain life equal to the damage prevented this way."
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Plains", 1);
    let strike = t.hand(P1, "Awe Strike");
    t.g.turn.priority = Some(P1);
    t.cast(P1, strike).target(a).go();
    t.resolve();
    // Two unblocked 5/5s: [10 damage] → [5 damage, gain 5] → [lose 5, gain 5]. That
    // wouldn't reduce the player's life below 1, so Worship doesn't apply.
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 2);
    assert!(!t.has_lost(P1));
}
