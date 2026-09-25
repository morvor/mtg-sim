//! CR 104: ending the game — winning, losing, draws, teams, the Emperor variant, the
//! limited range of influence option, and loops of mandatory actions.

use crate::r100_common::*;
use crate::r105_util::card_from_text;
use mtg_engine::game::{GameConfig, GameResult, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn win(ps: &[PlayerId]) -> Option<GameResult> {
    Some(GameResult::Win(ps.to_vec()))
}

/// P0 has 40 life and Felidar Sovereign ("At the beginning of your upkeep, if you have 40
/// or more life, you win the game."); advances to P0's next upkeep and resolves the
/// trigger.
fn felidar_upkeep(t: &mut TestGame) {
    t.battlefield(P0, "Felidar Sovereign");
    t.g.players[0].life = 40;
    let n = t.g.players.len();
    t.set_step(PlayerId((n - 1) as u8), Step::End);
    go_to(t, P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn an_effect_can_say_a_player_wins_and_the_game_ends_immediately() {
    cr!("104.1", "104.2", "104.2b");
    let mut t = TestGame::new(2);
    felidar_upkeep(&mut t);
    assert_eq!(t.g.result, win(&[P0]));
    assert!(t.g.player(P0).has_won);
    // The game is over: nothing else happens.
    let (turn, step) = (t.g.turn.number, t.g.turn.step);
    for _ in 0..10 {
        t.g.advance();
    }
    assert_eq!((t.g.turn.number, t.g.turn.step), (turn, step));
}

#[test]
fn the_rest_of_an_effect_does_nothing_once_the_game_has_ended() {
    // CR 104.1: the game ends immediately, even in the middle of a resolving spell.
    cr!("104.1", "104.3e");
    let mut t = TestGame::new(2);
    let c = card_from_text(
        "Sudden Ending",
        "{0}",
        "Sorcery",
        None,
        "Target player loses the game. You gain 5 life.",
    );
    let s = t.custom(P0, c, Zone::Hand(P0));
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.g.result, win(&[P0]));
    assert_eq!(t.life(P0), 20, "the game ended before the life gain");
}

#[test]
fn a_player_whose_opponents_all_left_wins_even_if_they_cant_win() {
    // CR 104.2a: this overrides effects that would preclude the player from winning.
    cr!("104.2a", "104.3a");
    ruling!("Abyssal Persecutor", "An opponent will lose a game if they concede");
    let mut t = TestGame::new(2);
    // "You can't win the game and your opponents can't lose the game."
    t.battlefield(P0, "Abyssal Persecutor");
    t.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
    assert_eq!(t.g.result, win(&[P0]));
}

#[test]
fn cant_win_and_cant_lose_effects_take_precedence() {
    // CR 101.2: "can't" beats an effect that says a player wins or loses.
    cr!("101.2", "104.2b", "104.3e");
    // Abyssal Persecutor: its controller doesn't win from Felidar Sovereign.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abyssal Persecutor");
    felidar_upkeep(&mut t);
    assert_eq!(t.g.result, None);
    // Platinum Angel: its controller's opponents can't win the game...
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Angel");
    felidar_upkeep(&mut t);
    assert_eq!(t.g.result, None);
    // ... and its controller can't lose the game (Door to Nothingness).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Angel");
    let door = t.battlefield(P0, "Door to Nothingness");
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, land, 2);
    }
    t.activate(P0, door, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(!t.has_lost(P1));
    assert_eq!(t.g.result, None);
}

#[test]
fn a_player_who_cant_lose_can_still_have_an_opponent_win() {
    cr!("104.2b");
    ruling!(
        "Lich's Mastery",
        "While you can’t lose the game, your opponents can still win the game if an effect says so."
    );
    let mut t = TestGame::new(2);
    let c = card_from_text(
        "Unlosing",
        "",
        "Enchantment",
        None,
        "You can't lose the game.",
    );
    t.custom(P1, c, Zone::Battlefield);
    felidar_upkeep(&mut t);
    assert_eq!(t.g.result, win(&[P0]));
}

#[test]
fn a_player_can_concede_even_while_they_cant_lose() {
    // CR 101.1: the only exception to cards overriding the rules is conceding.
    cr!("101.1", "104.3", "104.3a");
    ruling!(
        "Platinum Angel",
        "You can concede a game while Platinum Angel on the battlefield"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Platinum Angel");
    t.take_action(P0, Action::Concede);
    assert!(t.has_lost(P0));
    assert_eq!(t.g.result, win(&[P1]));
}

#[test]
fn a_player_who_concedes_leaves_a_multiplayer_game_immediately() {
    cr!("104.3a", "104.5");
    let mut t = TestGame::new(3);
    let bear = t.battlefield(P2, "Grizzly Bears");
    t.take_action(P2, Action::Concede);
    assert!(t.has_lost(P2));
    assert!(t.g.player(P2).left_game);
    // Its objects left with it (CR 800.4a); the game goes on.
    assert!(!t.g.is_live(bear) || t.zone(bear) != Zone::Battlefield);
    assert_eq!(t.g.result, None);
    assert_eq!(t.g.apnap(), vec![P0, P1]);
}

#[test]
fn zero_life_loses_the_next_time_a_player_would_receive_priority() {
    cr!("104.3b");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 3;
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.g.resolve_top();
    assert_eq!(t.life(P1), 0);
    // Not until state-based actions are checked.
    assert!(!t.has_lost(P1));
    t.settle();
    assert!(t.has_lost(P1));
    assert_eq!(t.g.result, win(&[P0]));
}

#[test]
fn drawing_from_an_empty_library_draws_the_rest_then_loses() {
    cr!("104.3c");
    let mut t = TestGame::new(2);
    let keep = t.g.players[1].library.split_off(29);
    t.g.players[1].library = keep;
    assert_eq!(t.library_size(P1), 1);
    let hand = t.hand_size(P1);
    t.g.draw_cards(P1, 3);
    assert_eq!(t.hand_size(P1), hand + 1, "drew the remaining card");
    assert!(!t.has_lost(P1));
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn ten_poison_counters_lose_the_game() {
    cr!("104.3d");
    let mut t = TestGame::new(2);
    t.g.players[1].counters.insert("poison".into(), 9);
    t.settle();
    assert!(!t.has_lost(P1));
    t.g.players[1].counters.insert("poison".into(), 10);
    assert!(!t.has_lost(P1));
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn an_effect_can_say_a_player_loses() {
    cr!("104.3", "104.3e");
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, "Door to Nothingness");
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, land, 2);
    }
    t.activate(P0, door, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.has_lost(P1));
    assert_eq!(t.g.result, win(&[P0]));
}

#[test]
fn all_players_losing_simultaneously_is_a_draw() {
    // Each player would win because their opponent left (CR 104.2a) at the same time as
    // they lose, so they lose (CR 104.3f); everyone lost at once: a draw (CR 104.4a).
    cr!("104.3f", "104.4", "104.4a");
    ruling!(
        "Incite Rebellion",
        "If this causes all players to have 0 or less life, the game is a draw."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].life = 4;
    t.g.players[1].life = 3;
    t.lands(P0, "Mountain", 2);
    // "Flame Rift deals 4 damage to each player."
    let rift = t.hand(P0, "Flame Rift");
    t.cast(P0, rift).go();
    t.resolve();
    assert!(t.has_lost(P0) && t.has_lost(P1));
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(!t.g.player(P0).has_won && !t.g.player(P1).has_won);
}

#[test]
fn players_who_win_and_lose_simultaneously_lose() {
    // "Each player wins the game": each player both wins and (because their opponent
    // wins) loses, so they lose — and the game is a draw.
    cr!("104.3f", "104.4a");
    let mut t = TestGame::new(2);
    let c = card_from_text(
        "Everyone Triumphs",
        "{0}",
        "Sorcery",
        None,
        "Each player wins the game.",
    );
    let s = t.custom(P0, c, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn an_effect_can_say_the_game_is_a_draw() {
    cr!("104.4", "104.4c");
    ruling!(
        "Divine Intervention",
        "When Divine Intervention’s third ability resolves, the game ends immediately. The game is a draw, meaning neither player wins and neither player loses."
    );
    ruling!(
        "Platinum Angel",
        "Effects that say the game is a draw, such as the _Legends_(TM) card Divine Intervention, are not affected by Platinum Angel."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Angel");
    let di = t.battlefield(P0, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 1);
    t.set_step(P1, Step::End);
    go_to(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert!(!t.g.player(P0).has_won && !t.g.player(P1).has_won);
}

#[test]
fn divine_intervention_needs_its_last_counter_removed() {
    cr!("104.4c");
    let mut t = TestGame::new(2);
    let di = t.battlefield(P0, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 2);
    t.set_step(P1, Step::End);
    go_to(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(di, "intervention"), 1);
    assert_eq!(t.g.result, None);
}

// ---------------------------------------------------------------------------
// Teams
// ---------------------------------------------------------------------------

#[test]
fn a_team_with_a_player_left_wins_and_all_its_players_win() {
    cr!("104.2c", "104.3g");
    let mut t = team_game(&[0, 0, 1, 1], GameConfig::default());
    // P0 loses first; its team is still in the game.
    t.take_action(P0, Action::Concede);
    assert_eq!(t.g.result, None);
    // P2 loses: team 1 still has P3.
    t.g.players[2].life = 0;
    t.settle();
    assert!(t.has_lost(P2));
    assert_eq!(t.g.result, None);
    // P3 loses: all players on team 1 have lost, so team 1 loses; team 0 wins — including
    // P0, who had lost.
    t.g.players[3].life = 0;
    t.settle();
    assert_eq!(t.g.result, win(&[P0, P1]));
    assert!(t.g.player(P0).has_won);
}

#[test]
fn all_remaining_teams_losing_simultaneously_is_a_draw() {
    cr!("104.4d");
    let mut t = team_game(&[0, 0, 1, 1], GameConfig::default());
    for p in 0..4 {
        t.g.players[p].life = 4;
    }
    t.lands(P0, "Mountain", 2);
    let rift = t.hand(P0, "Flame Rift");
    t.cast(P0, rift).go();
    t.resolve();
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn an_effect_saying_a_player_wins_makes_their_team_win() {
    cr!("104.2b", "104.2c");
    let mut t = team_game(&[0, 1, 0, 1], GameConfig::default());
    felidar_upkeep(&mut t);
    assert_eq!(t.g.result, win(&[P0, P2]));
    assert!(t.has_lost(P1) && t.has_lost(P3));
}

// ---------------------------------------------------------------------------
// Limited range of influence
// ---------------------------------------------------------------------------

fn ranged(n: usize, range: u32) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            range_of_influence: Some(range),
            ..Default::default()
        },
    )
}

#[test]
fn with_limited_range_a_win_makes_opponents_in_range_lose() {
    cr!("104.3h");
    ruling!(
        "Blood Tyrant",
        "if a spell or ability says that you win the game, it instead causes all of your opponents within your range of influence to lose the game"
    );
    let mut t = ranged(5, 1);
    // "Whenever a player loses the game, put five +1/+1 counters on this creature."
    let tyrant = t.battlefield(P2, "Blood Tyrant");
    felidar_upkeep(&mut t);
    // P1 and P4 are within one seat of P0; P2 and P3 aren't.
    assert!(t.has_lost(P1) && t.has_lost(P4()));
    assert!(!t.has_lost(P0) && !t.has_lost(P2) && !t.has_lost(P3));
    assert!(!t.g.player(P0).has_won);
    assert_eq!(t.g.result, None, "this may not end the game");
    t.resolve_all();
    assert_eq!(t.counters(tyrant, "+1/+1"), 10);
}

#[allow(non_snake_case)]
fn P4() -> PlayerId {
    PlayerId(4)
}

#[test]
fn with_limited_range_a_draw_is_a_draw_only_for_players_in_range() {
    cr!("104.4e", "104.5");
    ruling!(
        "Divine Intervention",
        "All players within range of Divine Intervention will leave the game."
    );
    let mut t = ranged(5, 1);
    let di = t.battlefield(P0, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 1);
    t.set_step(P4(), Step::End);
    go_to(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, None, "the game continues for the other players");
    for p in [P0, P1, P4()] {
        assert!(t.g.drew_game(p), "{p}");
        assert!(t.g.player(p).left_game);
        assert!(!t.has_lost(p));
    }
    assert_eq!(t.g.players_in_game(), vec![P2, P3]);
    // Divine Intervention left the game with its owner.
    assert!(t.g.find_in_zone(Zone::Battlefield, "Divine Intervention").is_empty());
}

// ---------------------------------------------------------------------------
// The Emperor variant
// ---------------------------------------------------------------------------

fn emperor_game() -> TestGame {
    team_game(
        &[0, 0, 0, 1, 1, 1],
        GameConfig {
            variant: Variant::Emperor,
            ..Default::default()
        },
    )
}

#[test]
fn emperors_sit_in_the_middle_of_their_teams() {
    cr!("104.3i");
    let t = emperor_game();
    assert_eq!(t.g.emperor_of(P0), Some(P1));
    assert_eq!(t.g.emperor_of(P3), Some(P4()));
    assert_eq!(t.g.range_of_influence(P1), Some(2));
    assert_eq!(t.g.range_of_influence(P0), Some(1));
}

#[test]
fn a_team_loses_when_its_emperor_loses_and_wins_with_its_emperor() {
    cr!("104.2d", "104.3i");
    let mut t = emperor_game();
    // A general of team 0 lost earlier.
    t.take_action(P2, Action::Concede);
    assert_eq!(t.g.result, None);
    // Team 1's emperor loses: its whole team loses.
    t.g.players[4].life = 0;
    t.settle();
    assert!(t.has_lost(P3) && t.has_lost(P4()) && t.has_lost(PlayerId(5)));
    // Team 0's emperor wins (its opponents all left), so its team wins.
    assert_eq!(t.g.result, win(&[P0, P1, P2]));
}

#[test]
fn the_game_is_a_draw_for_a_team_if_it_is_for_its_emperor() {
    cr!("104.4h", "104.4g");
    let mut t = emperor_game();
    // Team 0's general P0 (range 1) controls Divine Intervention: the game is a draw for
    // P0, P1 (team 0's emperor) and P5.
    let di = t.battlefield(P0, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 1);
    t.set_step(PlayerId(5), Step::End);
    go_to(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    // It's a draw for all of team 0 because it is for its emperor.
    for p in [P0, P1, P2] {
        assert!(t.g.drew_game(p), "{p}");
    }
    assert!(t.g.drew_game(PlayerId(5)));
    // Team 1 is the only team left and wins.
    assert_eq!(t.g.result, win(&[P3, P4()]));
}

// ---------------------------------------------------------------------------
// Commander damage
// ---------------------------------------------------------------------------

#[test]
fn twenty_one_combat_damage_from_one_commander_loses() {
    cr!("104.3j");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            starting_life: 40,
            ..Default::default()
        },
    );
    let cmdr = t.custom(
        P0,
        card_from_text("Big Commander", "", "Legendary Creature — Giant", Some((11, 11)), ""),
        Zone::Battlefield,
    );
    t.g.objects[cmdr.0 as usize].is_commander = true;
    let other = t.custom(
        P0,
        card_from_text("Big Friend", "", "Creature — Giant", Some((11, 11)), ""),
        Zone::Battlefield,
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(cmdr, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[],
    );
    t.settle();
    assert_eq!(t.life(P1), 18);
    assert!(!t.has_lost(P1), "only 11 damage from the commander so far");
    // Another 11 from the commander makes 22.
    t.g.objects[cmdr.0 as usize].tapped = false;
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(cmdr, Entity::Player(P1))]),
    );
    t.g.run_until(1000, |g| g.is_over() || g.turn.step == Step::EndOfCombat);
    assert_eq!(t.life(P1), 7);
    assert!(t.has_lost(P1));
    assert_eq!(t.g.result, win(&[P0]));
}

// ---------------------------------------------------------------------------
// Loops of mandatory actions
// ---------------------------------------------------------------------------

/// "Whenever a creature dies, [you may] return it to the battlefield." plus a 0/0 creature:
/// a loop that repeats forever.
fn loop_game(t: &mut TestGame, optional: bool) {
    let text = if optional {
        "Whenever a creature dies, you may return that card to the battlefield under its owner's control."
    } else {
        "Whenever a creature dies, return that card to the battlefield under its owner's control."
    };
    let e = card_from_text("Endless Return", "", "Enchantment", None, text);
    t.custom(P0, e, Zone::Battlefield);
    let z = card_from_text("Hollow Husk", "", "Creature — Construct", Some((0, 0)), "");
    t.custom(P0, z, Zone::Graveyard(P0));
    let husk = t.g.find_in_zone(Zone::Graveyard(P0), "Hollow Husk")[0];
    t.g.move_object(
        husk,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
}

#[test]
fn a_loop_of_mandatory_actions_is_a_draw() {
    cr!("104.4b");
    let mut t = TestGame::new(2);
    loop_game(&mut t, false);
    t.g.run_until(2000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(t.g.actions_taken < 200, "recognized quickly");
}

#[test]
fn a_loop_with_an_optional_action_is_not_a_draw() {
    cr!("104.4b");
    let mut t = TestGame::new(2);
    loop_game(&mut t, true);
    for _ in 0..8 {
        t.answer_yes(P0, true);
    }
    t.answer_yes(P0, false);
    t.g.run_until(400, |g| {
        g.is_over() || (g.stack.is_empty() && g.turn.step == Step::PostcombatMain)
    });
    assert_eq!(t.g.result, None);
    assert!(t.in_graveyard(P0, "Hollow Husk"));
}

#[test]
fn with_limited_range_a_loop_is_a_draw_for_players_involved_and_in_range() {
    cr!("104.4f");
    let mut t = ranged(5, 1);
    loop_game(&mut t, false);
    t.g.run_until(4000, |g| g.is_over() || !g.player(P0).in_game());
    for p in [P0, P1, P4()] {
        assert!(t.g.drew_game(p), "{p}");
    }
    assert!(t.g.player(P2).in_game() && t.g.player(P3).in_game());
    assert_eq!(t.g.result, None);
}
