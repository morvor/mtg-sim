//! Multiplayer judgment calls (docs/interpretations/players-multiplayer.md).

use super::r800_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

#[test]
fn the_attacking_player_is_an_active_player_its_controller_chooses() {
    cr!("506.2", "805.9");
    interp!("attacking-player-is-an-active-player");
    // Contested Game Ball: "Whenever you're dealt combat damage, the attacking player gains
    // control of this artifact and untaps it." P1's creature deals the damage; the Ball's
    // controller P2 chooses which attacking (active) player it refers to (CR 805.10c's
    // last sentence agrees here).
    let mut t = thg();
    let ball = t.battlefield(P2, "Contested Game Ball");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(giant, Entity::Player(P2))]);
    t.answer_choose(P2, &[Entity::Player(P1)]);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(
        last_entity_candidates(&t, P2),
        vec![Entity::Player(P0), Entity::Player(P1)]
    );
    assert_eq!(t.obj_now(ball).controller, P1);
}

#[test]
fn a_blockers_attacking_player_is_still_chosen_among_the_active_players() {
    // Not cited: CR 805.10c, which this test shows the engine doesn't follow yet.
    cr!("805.9", "810.9");
    interp!("attacking-player-is-an-active-player");
    // Souls of the Faultless: "Whenever this creature is dealt combat damage, you gain that
    // much life and the attacking player loses that much life." It blocks P1's Hill Giant.
    // CR 805.10c makes P1 the attacking player; the engine lets P2 choose between P0 and
    // P1 (a known deviation; in Two-Headed Giant the team's shared life total makes the
    // outcome the same).
    let mut t = thg();
    let souls = t.battlefield(P2, "Souls of the Faultless");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(giant, Entity::Player(P2))]);
    for p in [P2, P3] {
        t.answer(
            p,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(souls, giant)]),
        );
    }
    let life = t.life(P0);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(
        last_entity_candidates(&t, P2),
        vec![Entity::Player(P0), Entity::Player(P1)]
    );
    assert_eq!(t.life(P0), life - 3, "the team loses 3");
}
