//! Butcher Orgg (hand-written, `src/cards/butcher_orgg.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn unblocked_it_divides_damage_among_the_player_and_their_creatures() {
    cr!("510.1b");
    ruling!("Butcher Orgg", "You can use the ability to divide damage even if it is not blocked.");
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Butcher Orgg");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![4, 2]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(orgg, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn blocked_it_can_still_hit_the_player() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Butcher Orgg");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![6, 0]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(orgg, Entity::Player(P1))], &[(giant, orgg)]);
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(giant));
}

#[test]
fn declining_assigns_normally() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    let orgg = t.battlefield(P0, "Butcher Orgg");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, false);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(orgg, Entity::Player(P1))], &[(giant, orgg)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(giant));
}
