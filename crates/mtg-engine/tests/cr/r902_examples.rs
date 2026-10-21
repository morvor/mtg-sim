//! The worked examples of CR 902 (the Vanguard variant), played out as tests.

use crate::r100_common::{fillers, pregame};
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A two-player Vanguard game (not started): P0 plays `vanguard`, P1 none; 40-card decks.
fn vanguard_pregame(vanguard: &str, mulligans: bool) -> TestGame {
    let mut d0 = fillers(40);
    d0.push(card(vanguard));
    pregame(
        GameConfig {
            skip_mulligans: !mulligans,
            starting_player: Some(P0),
            ..GameConfig::vanguard_game()
        },
        vec![d0, fillers(40)],
    )
}

fn modifiers(t: &TestGame, p: PlayerId) -> (Option<i32>, Option<i32>) {
    let v = t.g.vanguard_of(p).expect("a vanguard");
    let c = &t.obj(v).chars;
    (c.hand_modifier, c.life_modifier)
}

#[test]
fn a_life_modifier_of_minus_3_makes_a_starting_life_total_of_17() {
    cr!("902.3");
    cr_example!("902.4", "life modifier of a player’s vanguard card is -3");
    // Mishra: hand modifier +0, life modifier −3.
    let mut t = vanguard_pregame("Mishra", false);
    t.g.start();
    assert_eq!(modifiers(&t, P0), (Some(0), Some(-3)));
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_hand_modifier_of_plus_2_draws_nine_and_three_mulligans_bottom_three() {
    cr!("902.5", "103.5");
    cr_example!("902.5a", "mulligan three times");
    // Volrath: hand modifier +2.
    let mut t = vanguard_pregame("Volrath", false);
    t.g.start();
    assert_eq!(modifiers(&t, P0).0, Some(2));
    assert_eq!(t.hand_size(P0), 9);
    // Three mulligans before keeping: a new hand of nine cards, three of them put on the
    // bottom of the library.
    let mut t = vanguard_pregame("Volrath", true);
    for _ in 0..3 {
        t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    }
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(false));
    t.g.start();
    let mulligans = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Mulligan { .. }))
        .count();
    assert_eq!(mulligans, 4, "three mulligans, then keep");
    let bottom: Vec<(usize, u32)> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::PutOnBottom { cards, n } if *p == P0 => Some((cards.len(), *n)),
            _ => None,
        })
        .collect();
    // Each mulligan draws a new hand of nine and puts as many cards on the bottom as
    // mulligans taken so far (CR 103.5): the third draws nine and puts three there.
    assert_eq!(bottom, vec![(9, 1), (9, 2), (9, 3)]);
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(t.library_size(P0), 34);
}

#[test]
fn a_hand_modifier_of_minus_1_makes_a_maximum_hand_size_of_six() {
    cr!("514.1");
    cr_example!("902.5b", "maximum hand size is six");
    // Ertai: hand modifier −1.
    let mut t = vanguard_pregame("Ertai", false);
    t.g.start();
    assert_eq!(modifiers(&t, P0).0, Some(-1));
    assert_eq!(t.player(P0).max_hand_size, Some(6));
    // More than six cards as the cleanup step begins: discard all but six.
    t.set_step(P0, Step::PrecombatMain);
    while t.hand_size(P0) < 9 {
        t.hand(P0, "Grizzly Bears");
    }
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(t.graveyard_size(P0), 3);
}
