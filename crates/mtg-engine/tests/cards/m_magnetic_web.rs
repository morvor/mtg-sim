//! Magnetic Web (hand-written, `src/cards/magnetic_web.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn magnet(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].counters.insert("magnet".into(), 1);
}

#[test]
fn one_magnetized_attacker_alone_is_illegal() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Magnetic Web");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    magnet(&mut t, a);
    magnet(&mut t, b);
    t.set_step(P0, Step::BeginningOfCombat);
    // Attacking with only one of them doesn't obey the other's requirement: the engine
    // declares a legal attack instead (here, none).
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn magnetized_creatures_attack_together() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Magnetic Web");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let _c = t.battlefield(P0, "Grizzly Bears");
    magnet(&mut t, a);
    magnet(&mut t, b);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn magnetized_creatures_block_a_magnetized_attacker() {
    cr!("509.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Magnetic Web");
    let a = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    magnet(&mut t, a);
    magnet(&mut t, giant);
    t.set_step(P0, Step::BeginningOfCombat);
    // P1 declares no blocks; the requirement makes the Giant block.
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(a));
}
