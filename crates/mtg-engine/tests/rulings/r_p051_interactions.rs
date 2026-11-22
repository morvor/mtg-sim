//! Rulings batch P051 — drain and draw interactions the rulings imply without spelling
//! out: a player who left the game loses no life (CR 800.4), losing and gaining life in
//! one effect before state-based actions are checked (CR 704.3, 608.2c), "your second card
//! each turn" with a draw replaced by dredge (CR 121.6, 614.6) or a draw-step draw
//! (CR 504.1), draw triggers under a "can't draw more than one card" effect (CR 121.2b) and
//! under draw doubling (CR 121.6), drains in Two-Headed Giant (CR 810.9), a drain trigger
//! whose source changed control (CR 603.3a), and a drain whose only target became illegal
//! (CR 608.2b).

use crate::r_p051_common::*;
use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::*;
use mtg_engine::decision::Action;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

#[test]
fn a_player_who_left_the_game_loses_no_life_to_a_drain() {
    cr!("104.3a", "800.4", "119.3");
    supported("Subversion");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Subversion");
    t.take_action(P2, Action::Concede);
    assert!(t.has_lost(P2));
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn losing_then_gaining_life_in_one_effect_happens_before_state_based_actions() {
    cr!("704.3", "608.2c", "704.5a");
    // Essence Harvest: "Target player loses X life and you gain X life, where X is the
    // greatest power among creatures you control." Targeting yourself at 3 life with a
    // 6-power creature: you go to -3 and back to 3 before state-based actions.
    supported("Essence Harvest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.g.players[0].life = 3;
    cast_pool(&mut t, P0, "Essence Harvest", &[pl(P0)]);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 3);
}

#[test]
fn second_card_each_turn_ignores_a_draw_replaced_by_dredge() {
    cr!("121.6", "614.6", "121.2", "702.52a");
    // Zimone and Dina: "Whenever you draw your second card each turn, target opponent
    // loses 2 life and you gain 2 life."
    supported("Zimone and Dina");
    supported("Golgari Brownscale");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zimone and Dina");
    t.graveyard(P0, "Golgari Brownscale");
    // The first draw is replaced by dredge 2: not a card drawn.
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    assert!(t.in_hand(P0, "Golgari Brownscale"));
    assert_eq!(drawn_this_turn(&t, P0), 0);
    t.clear_answers();
    t.answer_targets(P0, &[pl(P1)]);
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 0);
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn the_draw_step_draw_is_the_first_card_drawn() {
    cr!("504.1", "121.2");
    // Raven Eagle: "Whenever you draw your second card each turn, each opponent loses 1
    // life and you gain 1 life."
    supported("Raven Eagle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raven Eagle");
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(drawn_this_turn(&t, P0), 1);
    assert_eq!(lives(&t, 2), vec![20, 20]);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
}

#[test]
fn draw_triggers_count_only_the_draws_that_happen() {
    cr!("121.2b", "121.6", "603.2c");
    ruling!(
        "Underworld Dreams",
        "If multiple cards are drawn at once, this card triggers once for each card drawn."
    );
    // Under Leovold, an opponent told to draw three cards draws one: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Dreams");
    t.battlefield(P0, "Leovold, Emissary of Trest");
    draw(&mut t, P1, 3);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 1);
    // Under Thought Reflection, one draw instruction is two draws: two triggers.
    ruling!(
        "Niv-Mizzet, the Firemind",
        "If a spell or ability causes you to draw multiple cards, Niv-Mizzet's ability triggers that many times."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, the Firemind");
    t.battlefield(P0, "Thought Reflection");
    for _ in 0..2 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_fixed_gain_doesnt_depend_on_the_loss_but_life_lost_this_way_does() {
    cr!("119.3", "101.2", "702.101a");
    // Night Market Lookout ("each opponent loses 1 life and you gain 1 life") and Suffer
    // the Past ("For each card exiled this way, that player loses 1 life and you gain 1
    // life") against Platinum Emperion ("Your life total can't change."): you gain anyway.
    supported("Night Market Lookout");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Emperion");
    let l = t.battlefield(P0, "Night Market Lookout");
    attack_with(&mut t, &[(l, pl(P1))]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 20]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Emperion");
    let cards: Vec<Entity> = (0..2)
        .map(|_| obj(t.graveyard(P1, "Grizzly Bears")))
        .collect();
    crate::r_p046_common::fill_pool(&mut t, P0, 10);
    let s = t.hand(P0, "Suffer the Past");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &cards);
    t.cast(P0, s).x(2).go();
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![22, 20]);
    // Extort ("each opponent loses 1 life and you gain life equal to the total life lost
    // this way"): only the opponent who lost life counts.
    supported("Thrull Parasite");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Thrull Parasite");
    t.battlefield(P1, "Platinum Emperion");
    t.answer_yes(P0, true);
    cast_pool(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(lives(&t, 3), vec![21, 20, 19]);
}

#[test]
fn life_lost_this_turn_adds_up_over_separate_losses() {
    cr!("603.4", "119.3", "101.2");
    supported("Y'shtola, Night's Blessed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Y'shtola, Night's Blessed");
    for _ in 0..2 {
        t.g.lose_life(P1, 2);
        t.g.flush_events();
    }
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Life that can't be lost isn't lost: Gonti's Machinations under Platinum Emperion.
    supported("Gonti's Machinations");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gonti's Machinations");
    t.battlefield(P0, "Platinum Emperion");
    t.g.lose_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(crate::r_p044_common::player_counters(&t, P0, "energy"), 0);
}

/// P0 and P1 against P2 and P3, sharing team life totals of 30.
fn two_headed_giant() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

#[test]
fn drains_in_two_headed_giant_affect_each_opponent_and_the_team_total() {
    cr!("810.9", "810.9h", "119.3");
    // Subversion: each opponent (P2 and P3) loses 1 life, so their team loses 2; P0
    // gains the 2 life lost this way, and so does P0's team.
    supported("Subversion");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Subversion");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 4), vec![32, 32, 28, 28]);
    // Lampad of Death's Vigil: each opponent loses 1 life and you gain (only) 1 life.
    supported("Lampad of Death's Vigil");
    let mut t = two_headed_giant();
    let lampad = t.battlefield(P0, "Lampad of Death's Vigil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    assert!(activate_text(&mut t, P0, lampad, "Sacrifice", &[]));
    t.resolve_all();
    assert_eq!(lives(&t, 4), vec![31, 31, 28, 28]);
    // Platinum Emperion ("Your life total can't change.") under P3 protects P2 too:
    // nobody on that team loses life, so no life is lost this way and P0 gains none.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Subversion");
    t.battlefield(P3, "Platinum Emperion");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 4), vec![30, 30, 30, 30]);
}

#[test]
fn a_drain_trigger_stays_its_controllers_after_the_source_changes_control() {
    cr!("603.3a");
    // Subversion's upkeep trigger is P0's; P1 gains control of Subversion before it
    // resolves: P0's opponents (P1 and P2) lose life and P0 gains it.
    supported("Subversion");
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Subversion");
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    crate::r_s06_common::give_control(&mut t, s, P1);
    assert_eq!(t.obj_now(s).controller, P1);
    t.resolve_all();
    assert_eq!(lives(&t, 3), vec![22, 19, 19]);
}

#[test]
fn a_drain_whose_only_target_became_illegal_gains_nothing() {
    cr!("608.2b");
    // Archon of Cruelty: "Whenever this creature enters or attacks, target opponent
    // sacrifices a creature or planeswalker, discards a card, and loses 3 life. You draw a
    // card and gain 3 life." P1 gets hexproof (Leyline of Sanctity) in response.
    supported("Archon of Cruelty");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Archon of Cruelty");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses 3 life"), 1);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 20]);
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P1), 1);
}
