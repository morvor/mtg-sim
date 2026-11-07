//! Tahngarth, First Mate (hand-written, `src/cards/tahngarth_first_mate.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn joins_the_opponents_attack_until_end_of_combat() {
    cr!("508.4", "506.4");
    ruling!("Tahngarth, First Mate", "Tahngarth may end up attacking its owner this way");
    let mut t = TestGame::new(2);
    let tahn = t.battlefield(P0, "Tahngarth, First Mate");
    t.g.objects[tahn.0 as usize].tapped = true;
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.attack(&[(bear, Entity::Player(P0))], &[]);
    // Both dealt combat damage to P0; then control reverts.
    assert_eq!(t.life(P0), 13);
    t.advance_to_step(Step::PostcombatMain);
    assert_eq!(t.obj_now(tahn).controller, P0);
}

#[test]
fn not_while_untapped() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let tahn = t.battlefield(P0, "Tahngarth, First Mate");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.attack(&[(bear, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.obj_now(tahn).controller, P0);
}
