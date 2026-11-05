//! Rulings batch P051 — drain and draw interactions the rulings imply without spelling
//! out: a player who left the game loses no life (CR 800.4a), losing and gaining life in
//! one effect before state-based actions are checked (CR 704.3, 608.2c), "your second card
//! each turn" with a draw replaced by dredge (CR 121.6, 614.6) or a draw-step draw
//! (CR 504.1), draw triggers under a "can't draw more than one card" effect (CR 121.2b) and
//! under draw doubling (CR 121.6).

use crate::r_p051_common::*;
use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::*;
use mtg_engine::decision::Action;
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
    cr!("800.4a", "119.3");
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
    cr!("121.6", "614.6", "121.2");
    ruling!(
        "Zimone and Dina",
        "Zimone and Dina doesn’t need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when you draw your second card in a turn, that ability will trigger."
    );
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
    ruling!(
        "Raven Eagle",
        "The second triggered ability can trigger only once each turn. It doesn't matter whether Raven Eagle was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
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
    cr!("119.3", "119.8", "702.101a");
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
    cr!("603.4", "119.3");
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
