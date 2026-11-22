//! "Your first, second, or third turn of the game": the turns a player has taken
//! (`rule_statics::turns_taken`).

use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to `p`'s next precombat main phase.
fn next_main(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::Upkeep);
    t.advance_to(p, Step::PrecombatMain);
}

fn try_cast_avenger(t: &mut TestGame, p: PlayerId) -> bool {
    t.lands(p, "Plains", 2);
    let a = t.hand(p, "Serra Avenger");
    t.cast(p, a).try_go().is_ok()
}

#[test]
fn serra_avenger_cant_be_cast_during_your_first_three_turns() {
    cr!("601.3", "500.1");
    let mut t = TestGame::new(2);
    assert_eq!(t.g.player(P0).turns_taken, 1);
    assert!(!try_cast_avenger(&mut t, P0));
    next_main(&mut t, P0);
    assert_eq!(t.g.player(P0).turns_taken, 2);
    assert!(!try_cast_avenger(&mut t, P0));
    next_main(&mut t, P0);
    assert!(!try_cast_avenger(&mut t, P0));
    next_main(&mut t, P0);
    assert_eq!(t.g.player(P0).turns_taken, 4);
    assert!(try_cast_avenger(&mut t, P0));
}

#[test]
fn it_counts_the_turns_you_have_taken_not_the_games_turns() {
    cr!("601.3", "500.7");
    ruling!("Serra Avenger", "cares about how many turns you have taken");
    ruling!("Jace Reawakened", "cares about how many turns you have taken");
    let mut t = TestGame::new(2);
    // P1's third turn is the game's sixth turn: still too early for P1.
    next_main(&mut t, P1);
    next_main(&mut t, P1);
    next_main(&mut t, P1);
    assert_eq!(t.g.turn.number, 6);
    assert_eq!(t.g.player(P1).turns_taken, 3);
    assert!(!try_cast_avenger(&mut t, P1));
    // Jace Reawakened works the same way.
    t.lands(P1, "Island", 3);
    let jace = t.hand(P1, "Jace Reawakened");
    assert!(t.cast(P1, jace).try_go().is_err());
    // An extra turn is one of the turns P0 has taken: after Time Walk on P0's fourth turn,
    // the extra turn is P0's fifth.
    next_main(&mut t, P0);
    t.lands(P0, "Island", 2);
    let tw = t.hand(P0, "Time Walk");
    t.cast(P0, tw).go();
    t.resolve();
    next_main(&mut t, P0);
    assert_eq!(t.g.turn.number, 8);
    assert_eq!(t.g.player(P0).turns_taken, 5);
    // On P0's turns Jace may be cast.
    t.lands(P0, "Island", 3);
    let jace = t.hand(P0, "Jace Reawakened");
    assert!(t.cast(P0, jace).try_go().is_ok());
}

#[test]
fn it_can_be_cast_during_another_players_early_turn() {
    cr!("601.3");
    ruling!("Serra Avenger", "during another player's first, second, or third turns of the game");
    ruling!("Jace Reawakened", "during another player's first, second, or third turns of the game");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    t.advance_to(P1, Step::Upkeep);
    t.lands(P0, "Plains", 2);
    let a = t.hand(P0, "Serra Avenger");
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, a).try_go().is_ok());
}

#[test]
fn a_restarted_game_counts_turns_afresh() {
    cr!("727.1", "601.3");
    ruling!("Serra Avenger", "If the game is restarted (by Karn Liberated)");
    ruling!("Jace Reawakened", "If the game is restarted (by Karn Liberated)");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    for _ in 0..4 {
        next_main(&mut t, P0);
    }
    assert_eq!(t.g.player(P0).turns_taken, 5);
    let k = t.battlefield(P0, "Karn Liberated");
    t.g.objects[k.0 as usize]
        .counters
        .insert("loyalty".into(), 30);
    t.activate(P0, k, 2, &[]).unwrap();
    t.g.resolve_top();
    assert_eq!(t.g.turn.number, 1);
    next_main(&mut t, P0);
    assert_eq!(t.g.player(P0).turns_taken, 1);
    assert!(!try_cast_avenger(&mut t, P0));
}

#[test]
fn starting_town_enters_untapped_only_during_your_first_three_turns() {
    cr!("614.1c");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Starting Town");
    t.play_land(P0, a).unwrap();
    assert!(!t.obj_now(t.g.current(a)).tapped);
    // During P0's fourth turn it enters tapped.
    for _ in 0..3 {
        next_main(&mut t, P0);
    }
    let b = t.hand(P0, "Starting Town");
    t.play_land(P0, b).unwrap();
    assert!(t.obj_now(t.g.current(b)).tapped);
}
