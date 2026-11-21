//! Johan (hand-written, `src/cards/johan.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn your_creatures_attack_without_tapping_while_johan_is_untapped() {
    cr!("508.1f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Johan");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert!(!t.obj_now(bear).tapped);
}

#[test]
fn johan_cant_attack_if_you_do() {
    cr!("508.1c");
    let mut t = TestGame::new(2);
    let johan = t.battlefield(P0, "Johan");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.g.can_attack(johan));
}

#[test]
fn creatures_tap_if_you_dont() {
    cr!("508.1f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Johan");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert!(t.obj_now(bear).tapped);
}
