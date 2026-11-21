//! Rulings batch P038 — life totals that become a number of creatures (CR 119.5),
//! "each opponent loses" in Two-Headed Giant (CR 810.8), and opponents who left the
//! game (CR 800.4a).

use crate::r_p038_common::*;
use crate::r_s01_common::supported;
use mtg_engine::game::{GameConfig, GameResult, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 controls Shaman of Forgotten Ways and a Craw Wurm (total power 8) and mana for its
/// last ability.
fn shaman(t: &mut TestGame) -> ObjectId {
    supported("Shaman of Forgotten Ways");
    let s = t.battlefield(P0, "Shaman of Forgotten Ways");
    t.battlefield(P0, "Craw Wurm");
    mana_all(t, P0, 3);
    s
}

/// Activates the Shaman's life-total ability (its second activated ability).
fn activate_shaman(t: &mut TestGame, s: ObjectId) {
    t.activate(P0, s, 1, &[]).expect("activate");
}

#[test]
fn shaman_counts_each_players_creatures_as_it_resolves() {
    cr!("608.2h", "119.5");
    ruling!(
        "Shaman of Forgotten Ways",
        "Count the number of creatures each player controls when the last ability resolves to determine what number each player’s life total will become."
    );
    let mut t = TestGame::new(2);
    let s = shaman(&mut t);
    bears(&mut t, P1, 3);
    activate_shaman(&mut t, s);
    add(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 2);
    assert_eq!(t.life(P1), 4);
}

#[test]
fn becoming_a_life_total_is_gaining_or_losing_life() {
    cr!("119.5", "119.9");
    ruling!(
        "Shaman of Forgotten Ways",
        "For a player’s life total to become a specific number, the player gains or loses the appropriate amount of life."
    );
    ruling!(
        "Biorhythm",
        "This spell causes the player to lose or gain enough life to change to the new life total."
    );
    // Shaman: P1 at 12 with three creatures loses 9; P0 at 1 with three creatures (the
    // Shaman, the Wurm and Rhox Faithmender) gains 2, doubled by the Faithmender.
    let mut t = TestGame::new(2);
    let s = shaman(&mut t);
    t.battlefield(P0, "Rhox Faithmender");
    bears(&mut t, P1, 3);
    t.g.players[0].life = 1;
    t.g.players[1].life = 12;
    activate_shaman(&mut t, s);
    t.resolve_all();
    assert_eq!(t.life(P1), 3);
    assert_eq!(t.life(P0), 5, "gains 2, doubled to 4");

    // Biorhythm: P1 at 1 with two creatures gains 1 life — Ajani's Pridemate notices.
    supported("Biorhythm");
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P1, "Ajani's Pridemate");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.g.players[1].life = 1;
    cast_resolve(&mut t, P0, "Biorhythm", &[]);
    assert_eq!(t.life(P1), 2);
    assert_eq!(t.life(P0), 1, "loses 19");
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}

#[test]
fn shaman_with_no_creatures_left_sets_life_to_zero() {
    cr!("119.5", "704.5a", "104.4a");
    ruling!(
        "Shaman of Forgotten Ways",
        "If a player controls no creatures as the last ability resolves, their life total will become 0, causing that player to lose the game. If all players’ life totals become 0 this way, the game will be a draw."
    );
    // P1 has no creatures: P1 loses.
    let mut t = TestGame::new(2);
    let s = shaman(&mut t);
    activate_shaman(&mut t, s);
    t.resolve_all();
    assert_eq!(t.life(P1), 0);
    assert!(t.has_lost(P1) && !t.has_lost(P0));

    // Every creature dies in response: both players go to 0 and the game is a draw.
    let mut t = TestGame::new(2);
    let s = shaman(&mut t);
    activate_shaman(&mut t, s);
    for c in crate::r_s01_common::creatures(&t, P0) {
        kill(&mut t, c);
    }
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (0, 0));
    assert!(matches!(t.g.result, Some(GameResult::Draw)), "{:?}", t.g.result);
}

fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

#[test]
fn each_opponent_loses_twice_in_two_headed_giant() {
    cr!("810.8a", "810.9");
    ruling!(
        "Blight Pile",
        "In a Two-Headed Giant game, Blight Pile causes the opposing team to lose X life twice."
    );
    ruling!(
        "Gruesome Fate",
        "In a Two-Headed Giant game, Gruesome Fate causes the opposing team to lose 2 life for each creature you control."
    );
    ruling!(
        "Throne of the God-Pharaoh",
        "In a Two-Headed Giant game, the triggered ability of Throne of the God-Pharaoh causes the opposing team to lose life equal to twice the number of tapped creatures you control. Tapped creatures your teammate controls aren’t counted."
    );
    // Blight Pile: two creatures with defender (itself and Wall of Wood): 2 x 2.
    supported("Blight Pile");
    let mut t = two_headed_giant();
    let pile = t.battlefield(P0, "Blight Pile");
    t.battlefield(P0, "Wall of Wood");
    mana_all(&mut t, P0, 1);
    let before = t.life(P2);
    t.activate(P0, pile, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P2), before - 4);
    assert_eq!(t.life(P3), t.life(P2));

    // Gruesome Fate: three creatures -> the team loses 6.
    let mut t = two_headed_giant();
    bears(&mut t, P0, 3);
    bears(&mut t, P1, 1); // the teammate's creature isn't "you control"
    let before = t.life(P2);
    cast_resolve(&mut t, P0, "Gruesome Fate", &[]);
    assert_eq!(t.life(P2), before - 6);

    // Throne: P0 has two tapped creatures, the teammate one: the team loses 4.
    supported("Throne of the God-Pharaoh");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Throne of the God-Pharaoh");
    for b in bears(&mut t, P0, 2) {
        tap(&mut t, b);
    }
    bears(&mut t, P0, 1);
    let mate = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, mate);
    let before = t.life(P2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 4);
}

#[test]
fn opponents_who_left_the_game_are_not_counted() {
    cr!("800.4a");
    ruling!("Voice of Many", "Opponents who have left the game aren’t counted.");
    let mut t = TestGame::new(3);
    t.g.lose_game(P1);
    t.settle();
    assert!(t.has_lost(P1));
    let hand = t.hand_size(P0);
    enter(&mut t, P0, "Voice of Many");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "only P2 controls fewer creatures");
}
