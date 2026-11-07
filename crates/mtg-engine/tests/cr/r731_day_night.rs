//! CR 731.2: the untap step's check of the day/night designation.

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts Lightning Bolt at an opponent `n` times this turn.
fn cast_bolts(t: &mut TestGame, p: PlayerId, target: PlayerId, n: usize) {
    for _ in 0..n {
        t.lands(p, "Mountain", 1);
        let bolt = t.hand(p, "Lightning Bolt");
        t.cast(p, bolt).target(Entity::Player(target)).go();
        t.resolve_all();
    }
}

/// Advances to the untap step of the next turn (before its turn-based actions) and
/// returns the designation then, then to that turn's upkeep, returning the designation
/// after the untap step.
fn through_next_untap_step(t: &mut TestGame) -> (Option<bool>, Option<bool>) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(500, |g| {
        g.turn.number == turn + 1 && g.turn.step == Step::Untap && g.turn.stage == Stage::Begin
    });
    assert!(ok);
    let before = t.g.day;
    let ok = t.g.run_until(50, |g| g.turn.step == Step::Upkeep);
    assert!(ok);
    (before, t.g.day)
}

#[test]
fn day_becomes_night_if_the_previous_active_player_cast_no_spells() {
    cr!("731.2", "731.2a");
    ruling!(
        "Sunrise Cavalier",
        "If it is day, and the active player of the previous turn cast no spells during their turn, it becomes night."
    );
    ruling!(
        "Sunrise Cavalier",
        "Before a player untaps their permanents during the untap step, the game checks to see if the day/night designation should change."
    );
    let mut t = TestGame::new(2);
    t.g.set_day(true);
    // Another player's spell doesn't count.
    cast_bolts(&mut t, P1, P0, 1);
    assert_eq!(through_next_untap_step(&mut t), (Some(true), Some(false)));
}

#[test]
fn a_spell_cast_by_the_active_player_keeps_it_day() {
    cr!("731.2a");
    let mut t = TestGame::new(2);
    t.g.set_day(true);
    cast_bolts(&mut t, P0, P1, 1);
    assert_eq!(through_next_untap_step(&mut t), (Some(true), Some(true)));
}

#[test]
fn night_becomes_day_if_the_previous_active_player_cast_two_spells() {
    cr!("731.2", "731.2b");
    ruling!(
        "Sunrise Cavalier",
        "If it is night, and the active player of the previous turn cast two or more spells during their turn, it becomes day."
    );
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    cast_bolts(&mut t, P0, P1, 2);
    assert_eq!(through_next_untap_step(&mut t), (Some(false), Some(true)));
    // One spell isn't enough.
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    cast_bolts(&mut t, P0, P1, 1);
    assert_eq!(through_next_untap_step(&mut t), (Some(false), Some(false)));
}

#[test]
fn if_its_neither_day_nor_night_it_remains_neither() {
    cr!("731.2c");
    ruling!(
        "Sunrise Cavalier",
        "The game starts as neither."
    );
    let mut t = TestGame::new(2);
    assert_eq!(through_next_untap_step(&mut t), (None, None));
    let mut t = TestGame::new(2);
    cast_bolts(&mut t, P0, P1, 2);
    assert_eq!(through_next_untap_step(&mut t), (None, None));
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
fn with_shared_team_turns_any_spell_by_the_active_team_keeps_it_day() {
    cr!("731.2a");
    let mut t = two_headed_giant();
    t.g.set_day(true);
    // P1 is on the active team: its spell counts.
    cast_bolts(&mut t, P1, P2, 1);
    assert_eq!(through_next_untap_step(&mut t), (Some(true), Some(true)));
    // Next turn, nobody on the active team (P2 and P3) casts a spell: night.
    assert_eq!(t.g.turn.active, P2);
    assert_eq!(through_next_untap_step(&mut t), (Some(true), Some(false)));
}

#[test]
fn with_shared_team_turns_one_player_must_cast_two_spells_to_make_it_day() {
    cr!("731.2b");
    let mut t = two_headed_giant();
    t.g.set_day(false);
    // Each player on the active team casts one spell: it stays night.
    cast_bolts(&mut t, P0, P2, 1);
    cast_bolts(&mut t, P1, P2, 1);
    assert_eq!(through_next_untap_step(&mut t), (Some(false), Some(false)));
    // A player of the active team casts two: it becomes day.
    assert_eq!(t.g.turn.active, P2);
    t.set_step(P2, Step::PrecombatMain);
    cast_bolts(&mut t, P3, P0, 2);
    assert_eq!(through_next_untap_step(&mut t), (Some(false), Some(true)));
}
