//! CR 507: beginning of combat step.

use crate::r506_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn two_player_game_opponent_automatically_defends() {
    cr!("507.1");
    let mut t = TestGame::new(2);
    go_to(&mut t, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P1]);
    // No choice was needed and nothing used the stack.
    assert_eq!(
        count_asked(&t, P0, |d| matches!(d, Decision::ChooseEntities { .. })),
        0
    );
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn defending_player_choice_doesnt_use_the_stack() {
    cr!("507.1");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: false,
            ..Default::default()
        },
    );
    t.answer_choose(P0, &[Entity::Player(P1)]);
    go_to(&mut t, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P1]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn active_player_gets_priority_in_beginning_of_combat() {
    cr!("507.2");
    let mut t = TestGame::new(2);
    let order = priority_order_in(&mut t, Step::BeginningOfCombat);
    assert_eq!(order, vec![P0, P1]);
    // On P1's turn, P1 gets priority first.
    t.advance_to(P1, Step::Upkeep);
    let order = priority_order_in(&mut t, Step::BeginningOfCombat);
    assert_eq!(order, vec![P1, P0]);
}
