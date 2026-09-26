//! "You don't lose the game for having 0 or less life." (compiled by
//! `oracle/patterns/misc_game_rules_life.rs`): the state-based action of CR 704.5a
//! doesn't apply to that player.

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::*;

fn ability_supported(name: &str, line: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().iter().all(|u| !u.contains(line)),
        "{name}: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn a_player_at_zero_or_less_life_doesnt_lose() {
    cr!("704.5a", "119.6", "104.3b");
    ruling!(
        "Lich",
        "You can lose life and take damage, and thereby have a negative life total"
    );
    ability_supported("Lich", "don't lose the game");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lich");
    t.settle();
    t.g.lose_life(P0, 25);
    t.settle();
    assert_eq!(t.life(P0), -5);
    assert!(!t.has_lost(P0));
    // The opponent isn't protected.
    t.g.lose_life(P1, 20);
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn the_player_still_loses_for_other_reasons() {
    cr!("704.5a", "704.5c");
    ruling!(
        "Phyrexian Unlife",
        "You can still lose the game for other reasons, including having ten or more poison counters"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Unlife");
    t.settle();
    t.g.lose_life(P0, 20);
    t.settle();
    assert!(!t.has_lost(P0));
    t.g.players[P0.idx()].counters.insert("poison".into(), 10);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn losing_the_permanent_at_zero_life_loses_the_game() {
    cr!("704.5a");
    ruling!("Phyrexian Unlife", "Phyrexian Unlife leaves the battlefield, you");
    let mut t = TestGame::new(2);
    let unlife = t.battlefield(P0, "Phyrexian Unlife");
    t.settle();
    t.g.lose_life(P0, 20);
    t.settle();
    assert!(!t.has_lost(P0));
    t.g.destroy(unlife, None);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn no_maximum_hand_size_and_no_loss_for_zero_life() {
    cr!("704.5a", "402.2");
    ability_supported("Marina Vendrell's Grimoire", "maximum hand size");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Marina Vendrell's Grimoire");
    t.settle();
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    assert_eq!(t.g.player(P1).max_hand_size, Some(7));
    t.g.lose_life(P0, 20);
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn a_two_headed_giant_team_still_loses_if_only_one_player_is_protected() {
    cr!("810.8a", "810.8c", "704.6a");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P2, "Phyrexian Unlife");
    t.settle();
    t.g.players[2].life = 1;
    t.g.players[3].life = 1;
    t.g.lose_life(P2, 1);
    t.settle();
    // P3 has 0 or less life and loses, so the whole team does.
    assert!(t.has_lost(P2) && t.has_lost(P3));

    // With both players protected, the team doesn't lose.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P2, "Phyrexian Unlife");
    t.battlefield(P3, "Phyrexian Unlife");
    t.settle();
    t.g.players[2].life = 1;
    t.g.players[3].life = 1;
    t.g.lose_life(P2, 1);
    t.settle();
    assert!(!t.has_lost(P2) && !t.has_lost(P3));
}
