//! Defensive Formation (hand-written, `src/cards/defensive_formation.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_defending_player_assigns_the_attackers_damage() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Defensive Formation");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    // All of it on the first blocker, regardless of lethal damage.
    t.answer(P1, DecisionKind::Damage, Answer::Numbers(vec![6, 0]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wurm, Entity::Player(P1))], &[(b1, wurm), (b2, wurm)]);
    assert!(!t.on_battlefield(b1));
    assert!(t.on_battlefield(b2));
}

#[test]
fn its_controllers_own_attackers_are_unaffected() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defensive Formation");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wurm, Entity::Player(P1))], &[(b1, wurm), (b2, wurm)]);
    // The attacking player's default assignment kills both.
    assert!(!t.on_battlefield(b1));
    assert!(!t.on_battlefield(b2));
}
