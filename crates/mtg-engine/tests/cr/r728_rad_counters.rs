//! CR 728: rad counters.

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts cards on top of `p`'s library (the last one ends up on top).
fn stack_library(t: &mut TestGame, p: PlayerId, names: &[&str]) {
    for n in names {
        t.library_top(p, n);
    }
}

fn rad(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::RAD)
}

#[test]
fn the_rad_ability_mills_and_drains_the_player_whose_main_phase_it_is() {
    cr!("728.1");
    ruling!(
        "Strong, the Brutish Thespian",
        "There is an inherent triggered ability associated with having rad counters. This triggered ability has no source and is controlled by the active player."
    );
    ruling!(
        "Strong, the Brutish Thespian",
        "Rad counters don’t go away as steps, phases, or turns end."
    );
    let mut t = TestGame::new(2);
    // P0's next draw, then (from the top) a creature, a land and a creature.
    stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Forest", "Hill Giant", "Lightning Bolt"],
    );
    t.g.add_counters(Entity::Player(P0), counters::RAD, 3, None);
    t.g.add_counters(Entity::Player(P1), counters::RAD, 2, None);
    // Not at the beginning of another player's precombat main phase.
    t.advance_to(P1, Step::PrecombatMain);
    t.settle();
    let mine: Vec<PlayerId> = t
        .g
        .stack
        .iter()
        .map(|s| t.g.obj(*s).controller)
        .collect();
    assert_eq!(mine, vec![P1]);
    t.resolve_all();
    assert_eq!(rad(&t, P0), 3);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let ability = t.g.stack[0];
    assert_eq!(t.g.obj(ability).controller, P0);
    t.resolve();
    // Three cards milled, two of them nonland: 2 life lost and 2 rad counters removed.
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.life(P0), 18);
    assert_eq!(rad(&t, P0), 1);
}

#[test]
fn with_shared_team_turns_each_active_player_with_rad_counters_is_affected() {
    cr!("728.1");
    ruling!(
        "Strong, the Brutish Thespian",
        "In a game using the shared team turns option, such as an Archenemy or Two-Headed Giant game, the inherent triggered ability associated with rad counters triggers once for each player on the active team that has rad counters. Each instance of that ability is controlled by one of those players."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.g.add_counters(Entity::Player(P0), counters::RAD, 1, None);
    t.g.add_counters(Entity::Player(P1), counters::RAD, 1, None);
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    let mut controllers: Vec<PlayerId> = t
        .g
        .stack
        .iter()
        .map(|s| t.g.obj(*s).controller)
        .collect();
    controllers.sort();
    assert_eq!(controllers, vec![P0, P1]);
    t.resolve_all();
    // Each milled a (nonland) filler card.
    assert_eq!(rad(&t, P0), 0);
    assert_eq!(rad(&t, P1), 0);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn gaining_life_rather_than_losing_life_from_radiation() {
    cr!("728.1a");
    ruling!(
        "Strong, the Brutish Thespian",
        "You’ll still mill cards equal to the number of rad counters you have, and you’ll still remove a rad counter from yourself for each nonland card you milled, but instead of losing life, you’ll gain 1 life for each nonland card you milled."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Strong, the Brutish Thespian");
    stack_library(&mut t, P0, &["Grizzly Bears", "Forest", "Hill Giant"]);
    t.g.add_counters(Entity::Player(P0), counters::RAD, 3, None);
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(rad(&t, P0), 1);
    // Other life loss isn't from radiation.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}
