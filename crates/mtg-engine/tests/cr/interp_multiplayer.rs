//! Multiplayer judgment calls (docs/interpretations/players-multiplayer.md), and "the
//! attacking player" with shared team turns (CR 805.10c).

use super::r506_common::{bf, custom_card};
use super::r800_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

/// How many times `p` was asked to choose among entities (e.g. an attacking player).
fn entity_choices(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::ChooseEntities { .. }))
        .count()
}

/// Life `p` lost this turn.
fn life_lost(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.life_lost.get(&p).copied().unwrap_or(0)
}

/// P2 and P3 block as `blocks` (the defending team's one combined block).
fn team_blocks(t: &mut TestGame, blocks: &[(ObjectId, ObjectId)]) {
    for p in [P2, P3] {
        t.answer(p, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
}

#[test]
fn the_attacking_player_without_a_blocker_is_the_one_its_controller_chooses() {
    cr!("506.2", "805.10a", "805.10c");
    interp!("attacking-player-with-shared-team-turns");
    // Contested Game Ball: "Whenever you're dealt combat damage, the attacking player gains
    // control of this artifact and untaps it." No blocking creature is involved: P1's
    // creature deals the damage, and the Ball's controller P2 chooses one of the
    // attacking players (P0 and P1 both are, CR 805.10a).
    let mut t = thg();
    let ball = t.battlefield(P2, "Contested Game Ball");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(giant, Entity::Player(P2))]);
    t.answer_choose(P2, &[Entity::Player(P0)]);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(
        last_entity_candidates(&t, P2),
        vec![Entity::Player(P0), Entity::Player(P1)]
    );
    assert_eq!(t.obj_now(ball).controller, P0);
}

#[test]
fn a_blocking_creatures_attacking_player_is_the_controller_of_what_it_blocks() {
    cr!("805.10c", "810.9");
    interp!("attacking-player-with-shared-team-turns");
    // Souls of the Faultless: "Whenever this creature is dealt combat damage, you gain
    // that much life and attacking player loses that much life." It blocks P1's Craw Wurm
    // (6/4) and dies: the attacking player is P1, the controller of the creature it was
    // blocking, with no choice for P2 (even though it has left the battlefield).
    let mut t = thg();
    let souls = t.battlefield(P2, "Souls of the Faultless");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(wurm, Entity::Player(P2))]);
    team_blocks(&mut t, &[(souls, wurm)]);
    let life = t.life(P0);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert!(!on_bf(&t, souls), "Souls died");
    assert_eq!(entity_choices(&t, P2), 0, "no choice of attacking player");
    assert_eq!(life_lost(&t, P1), 6);
    assert_eq!(life_lost(&t, P0), 0);
    // The team shares its life total (CR 810.9a).
    assert_eq!(t.life(P0), life - 6);
}

#[test]
fn a_blocker_blocking_both_attacking_players_creatures_lets_its_controller_choose() {
    cr!("805.10c");
    interp!("attacking-player-with-shared-team-turns");
    // A Souls of the Faultless that can block two creatures blocks one of P0's and one of
    // P1's. Either attacking player could be meant, so P2 chooses one of them.
    let wide_souls = || {
        custom_card(
            "Wide Souls",
            "Creature — Spirit",
            Some((0, 10)),
            "This creature can block an additional creature each combat.\nWhenever this creature is dealt combat damage, you gain that much life and attacking player loses that much life.",
        )
    };
    for chosen in [P0, P1] {
        let mut t = thg();
        let souls = bf(&mut t, P2, wide_souls());
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.set_step(P0, Step::BeginningOfCombat);
        declare(
            &mut t,
            &[(bears, Entity::Player(P2)), (giant, Entity::Player(P2))],
        );
        team_blocks(&mut t, &[(souls, bears), (souls, giant)]);
        t.answer_choose(P2, &[Entity::Player(chosen)]);
        go_to(&mut t, Step::CombatDamage);
        t.resolve_all();
        assert_eq!(
            last_entity_candidates(&t, P2),
            vec![Entity::Player(P0), Entity::Player(P1)]
        );
        assert_eq!(life_lost(&t, chosen), 5, "chosen: {chosen:?}");
        let other = if chosen == P0 { P1 } else { P0 };
        assert_eq!(life_lost(&t, other), 0, "chosen: {chosen:?}");
    }
}

#[test]
fn each_blocking_creature_determines_its_own_attacking_player() {
    cr!("805.10c");
    interp!("attacking-player-with-shared-team-turns");
    // Two Souls of the Faultless: one blocks P0's Hill Giant, the other P1's Grizzly
    // Bears. Each one's ability refers to the controller of what that Souls blocks.
    let mut t = thg();
    let souls_a = t.battlefield(P2, "Souls of the Faultless");
    let souls_b = t.battlefield(P3, "Souls of the Faultless");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(giant, Entity::Player(P2)), (bears, Entity::Player(P3))],
    );
    team_blocks(&mut t, &[(souls_a, giant), (souls_b, bears)]);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(entity_choices(&t, P2) + entity_choices(&t, P3), 0);
    assert_eq!(life_lost(&t, P0), 3);
    assert_eq!(life_lost(&t, P1), 2);
}

#[test]
fn norns_decree_draws_for_the_attacking_player_its_controller_chooses() {
    cr!("805.10c");
    interp!("attacking-player-with-shared-team-turns");
    // Norn's Decree: "Whenever a player attacks, if one or more players being attacked are
    // poisoned, the attacking player draws a card." No blocking creature: P2 chooses
    // which attacking player draws.
    for chosen in [P0, P1] {
        let mut t = thg();
        t.battlefield(P2, "Norn's Decree");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.add_counters(Entity::Player(P2), "poison", 1, None);
        t.set_step(P0, Step::BeginningOfCombat);
        let hands = (t.hand_size(P0), t.hand_size(P1));
        declare(&mut t, &[(bears, Entity::Player(P2))]);
        t.answer_choose(P2, &[Entity::Player(chosen)]);
        go_to(&mut t, Step::DeclareAttackers);
        t.resolve_all();
        assert_eq!(
            last_entity_candidates(&t, P2),
            vec![Entity::Player(P0), Entity::Player(P1)]
        );
        let drew = (
            t.hand_size(P0) as i64 - hands.0 as i64,
            t.hand_size(P1) as i64 - hands.1 as i64,
        );
        assert_eq!(drew, if chosen == P0 { (1, 0) } else { (0, 1) });
    }
}

#[test]
fn invasion_plans_controller_chooses_which_attacking_player_declares_blocks() {
    cr!("805.10c", "805.10d");
    interp!("attacking-player-with-shared-team-turns");
    // Invasion Plans: "The attacking player chooses how each creature blocks each combat."
    // With two attacking players, its controller P2 chooses one; P1 then declares the
    // defending team's blocks.
    let mut t = thg();
    t.battlefield(P2, "Invasion Plans");
    let a0 = t.battlefield(P0, "Grizzly Bears");
    let a1 = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P3, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(a0, Entity::Player(P2)), (a1, Entity::Player(P3))],
    );
    t.answer_choose(P2, &[Entity::Player(P1)]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, a1)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(
        last_entity_candidates(&t, P2),
        vec![Entity::Player(P0), Entity::Player(P1)]
    );
    let deciders: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::DeclareBlockers { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(deciders, vec![P1]);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blocking(wall), vec![a1]);
}

#[test]
fn without_shared_team_turns_the_attacking_player_is_the_active_player() {
    cr!("506.2");
    interp!("attacking-player-with-shared-team-turns");
    // A three-player Free-for-All game: no team turns, so Souls of the Faultless's
    // attacking player is the active player, with no choice.
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..GameConfig::default()
        },
    );
    let souls = t.battlefield(P1, "Souls of the Faultless");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(souls, giant)]),
    );
    let life = t.life(P0);
    go_to(&mut t, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(entity_choices(&t, P1), 0);
    assert_eq!(t.life(P0), life - 3);
    assert_eq!(t.life(P1), 23);
}

#[test]
fn norns_decree_triggers_once_for_each_opponent_whose_creatures_dealt_damage() {
    cr!("603.2c", "805.10a", "810.10");
    interp!("one-or-more-an-opponent-controls-per-opponent");
    // Norn's Decree: "Whenever one or more creatures an opponent controls deal combat
    // damage to you, that opponent gets a poison counter." Both attacking players' creatures
    // deal P2 combat damage at once: it triggers once for each of them, and each gets a
    // poison counter (individually, CR 810.10).
    let mut t = thg();
    t.battlefield(P2, "Norn's Decree");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (bears, Entity::Player(P2)),
            (elves, Entity::Player(P2)),
            (giant, Entity::Player(P2)),
        ],
    );
    // Stop as the Decree's triggers are put on the stack (nothing else triggers).
    t.g.run_until(10_000, |g| !g.stack.is_empty() || g.turn.step == Step::End);
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.g.player(P0).poison(), 1, "{}", t.dump_log());
    assert_eq!(t.g.player(P1).poison(), 1);
    assert_eq!(t.g.player(P2).poison(), 0);
    assert_eq!(t.g.player(P3).poison(), 0);
}
