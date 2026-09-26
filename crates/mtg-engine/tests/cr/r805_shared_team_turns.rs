//! CR 805: the shared team turns option.

use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

fn thg_pregame(starting: PlayerId, decks: Vec<Vec<std::sync::Arc<mtg_engine::card::CardDef>>>) -> TestGame {
    super::r100_common::pregame(
        GameConfig {
            starting_player: Some(starting),
            ..GameConfig::two_headed_giant(vec![0, 0, 1, 1])
        },
        decks,
    )
}

fn decisions_of(t: &TestGame, f: impl Fn(&Decision) -> bool) -> Vec<PlayerId> {
    t.asked()
        .into_iter()
        .filter(|(_, d)| f(d))
        .map(|(p, _)| p)
        .collect()
}

#[test]
fn shared_team_turns_needs_teams_seated_together() {
    cr!("805.1");
    // Always used in Two-Headed Giant.
    assert!(thg().g.uses_shared_team_turns());
    // Other team games may use it, if each team's members sit in adjacent seats.
    let tvt = GameConfig {
        shared_team_turns: true,
        ..GameConfig::team_vs_team(vec![0, 0, 0, 1, 1, 1])
    };
    assert_eq!(tvt.validate(6), Ok(()));
    assert!(TestGame::with_config(6, tvt).g.uses_shared_team_turns());
    let apart = GameConfig {
        shared_team_turns: true,
        ..GameConfig::default()
    };
    let apart = GameConfig {
        teams: Some(vec![0, 1, 0, 1]),
        ..apart
    };
    assert!(apart
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::TeamsNotTogether));
    let t = TestGame::with_config(4, apart);
    assert!(!t.g.uses_shared_team_turns());
    // Without teams there's nothing to share.
    let solo = GameConfig {
        shared_team_turns: true,
        ..GameConfig::free_for_all()
    };
    assert!(solo
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::SharedTeamTurnsWithoutTeams));
}

#[test]
fn the_primary_player_sits_rightmost_and_decides_for_the_team() {
    cr!("805.2");
    // Seats 3 and 0 form one team (around the table), seats 1 and 2 the other. From each
    // team's side of the table its rightmost seat is the one whose right-hand neighbor
    // (the previous seat in turn order) isn't a teammate.
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![1, 0, 0, 1]));
    assert_eq!(t.g.primary_player(P0), P3);
    assert_eq!(t.g.primary_player(P3), P3);
    assert_eq!(t.g.primary_player(P2), P1);
    // The team's combined attack and block are declared by its primary player.
    let a0 = bear(&mut t, P0);
    let a3 = bear(&mut t, P3);
    let b2 = bear(&mut t, P2);
    t.set_step(P3, Step::BeginningOfCombat);
    declare(&mut t, &[(a0, Entity::Player(P1)), (a3, Entity::Player(P2))]);
    block(&mut t, P1, &[(b2, a0)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(
        decisions_of(&t, |d| matches!(d, Decision::DeclareAttackers { .. })),
        vec![P3]
    );
    assert_eq!(
        decisions_of(&t, |d| matches!(d, Decision::DeclareBlockers { .. })),
        vec![P1]
    );
    assert!(is_blocked(&t, a0));
}

#[test]
fn the_starting_team_is_chosen_like_a_starting_player() {
    cr!("805.3");
    let mut t = super::r100_common::pregame(
        GameConfig {
            first_turn_chooser: Some(P2),
            skip_mulligans: true,
            ..GameConfig::two_headed_giant(vec![0, 0, 1, 1])
        },
        (0..4).map(|_| super::r100_common::fillers(40)).collect(),
    );
    t.answer_choose(P2, &[Entity::Player(P0)]);
    t.g.start();
    // The chooser picked among the teams.
    let offered = last_entity_candidates(&t, P2);
    assert_eq!(offered.len(), 2);
    assert_eq!(t.g.start.starting_team, Some(0));
    assert_eq!(t.g.turn.starting_player, P0);
}

#[test]
fn the_starting_team_declares_mulligans_first() {
    cr!("805.3a");
    let mut t = thg_pregame(P2, (0..4).map(|_| super::r100_common::fillers(40)).collect());
    // P2 keeps; P3 may still mulligan after that.
    t.answer(P3, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    let order = decisions_of(&t, |d| matches!(d, Decision::Mulligan { .. }));
    assert_eq!(&order[..4], &[P2, P3, P0, P1]);
    assert_eq!(t.g.player(P3).mulligans, 1);
    assert_eq!(t.g.player(P2).mulligans, 0);
}

#[test]
fn the_starting_team_puts_opening_hand_cards_onto_the_battlefield_first() {
    cr!("805.3b");
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P2),
            skip_mulligans: true,
            ..GameConfig::two_headed_giant(vec![0, 0, 1, 1])
        },
        (0..4)
            .map(|_| super::r100_common::copies("Leyline of Sanctity", 20))
            .collect(),
    );
    t.g.start();
    let mut order: Vec<PlayerId> = Vec::new();
    for (p, d) in t.asked() {
        if matches!(d, Decision::YesNo { .. }) && order.last() != Some(&p) {
            order.push(p);
        }
    }
    assert_eq!(order, vec![P2, P3, P0, P1]);
    assert!(!t.named_on_battlefield("Leyline of Sanctity").is_empty());
}

#[test]
fn teams_take_turns() {
    cr!("805.4", "805.4a");
    let mut t = thg();
    // It's P0's team's turn: P0 and P1 are the active team, P2 and P3 the nonactive team.
    assert!(t.g.is_active_player(P0) && t.g.is_active_player(P1));
    assert!(!t.g.is_active_player(P2) && !t.g.is_active_player(P3));
    assert_eq!(t.g.active_players(), vec![P0, P1]);
    // The next turn is the other team's, then this team's again.
    to_turn_of(&mut t, P2);
    assert!(t.g.is_active_player(P3) && !t.g.is_active_player(P1));
    to_turn_of(&mut t, P0);
    assert_eq!(t.g.turn.number, 3);
}

#[test]
fn each_player_on_the_team_draws_and_may_play_a_land() {
    cr!("805.4b", "805.4c");
    let mut t = thg();
    t.set_step(P1, Step::End);
    let before = (t.hand_size(P2), t.hand_size(P3), t.hand_size(P0));
    to_step(&mut t, P2, Step::PrecombatMain);
    assert_eq!(t.hand_size(P2), before.0 + 1);
    assert_eq!(t.hand_size(P3), before.1 + 1);
    assert_eq!(t.hand_size(P0), before.2);
    // Each of them may play a land this turn.
    let f2 = t.hand(P2, "Forest");
    let f3 = t.hand(P3, "Forest");
    assert!(t.play_land(P2, f2).is_ok());
    assert!(t.play_land(P3, f3).is_ok());
    let again = t.hand(P3, "Forest");
    assert!(t.play_land(P3, again).is_err(), "one land each");
}

#[test]
fn each_players_step_triggers_trigger_for_each_player_on_the_active_team() {
    cr!("805.4d");
    ruling!(
        "Crescendo of War",
        "the first ability triggers once during each team's upkeep"
    );
    // Howling Mine: "At the beginning of each player's draw step, if this artifact is
    // untapped, that player draws an additional card." It refers to "that player": it
    // triggers once for each player on the active team.
    let mut t = thg();
    t.battlefield(P0, "Howling Mine");
    // Crescendo of War: "At the beginning of each upkeep, put a strife counter on this
    // enchantment." It triggers once per team upkeep.
    let crescendo = t.battlefield(P0, "Crescendo of War");
    t.set_step(P1, Step::End);
    let before = (t.hand_size(P2), t.hand_size(P3));
    to_step(&mut t, P2, Step::PrecombatMain);
    assert_eq!(t.hand_size(P2), before.0 + 2);
    assert_eq!(t.hand_size(P3), before.1 + 2);
    assert_eq!(t.counters(crescendo, "strife"), 1);
}

#[test]
fn teams_have_priority() {
    cr!("805.5", "805.5a", "805.5b");
    let mut t = thg();
    let a = t.custom(P1, super::r114_common::free_instant("Team Gift"), Zone::Hand(P1));
    let b = t.custom(P2, super::r114_common::free_instant("Other Gift"), Zone::Hand(P2));
    let cast = |c: ObjectId| Action::Cast {
        card: c,
        method: mtg_engine::object::CastMethod::Normal,
    };
    // P0's team has priority: P1 may cast a spell; the other team may not.
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(t.g.has_priority(P1));
    assert!(t.g.perform_action(P2, cast(b)).is_err());
    t.g.take_action(P1, cast(a));
    assert_eq!(t.stack_len(), 1);
    // Every player of the team passing makes the team pass; the other team receives
    // priority. The spell resolves only once all teams have passed in succession.
    t.g.take_action(P1, Action::Pass);
    assert!(t.g.has_priority(P2) && t.g.has_priority(P3));
    t.g.take_action(P2, Action::Pass);
    t.g.take_action(P3, Action::Pass);
    assert_eq!(t.stack_len(), 1);
    t.g.take_action(P0, Action::Pass);
    assert_eq!(t.stack_len(), 0);
    // Then the active team receives priority.
    assert!(t.g.has_priority(P0) && t.g.has_priority(P1));
    // With an empty stack, all teams passing ends the step.
    for p in [P0, P1, P2, P3] {
        t.g.take_action(p, Action::Pass);
    }
    assert_eq!(t.g.turn.stage, mtg_engine::turn::Stage::End);
}

#[test]
fn the_active_team_makes_simultaneous_choices_first() {
    cr!("805.6");
    // Innocent Blood: "Each player sacrifices a creature." It's P1's team's turn (P1 is
    // representing it here): P1 and P0 choose before P2 and P3.
    let mut t = thg();
    t.set_step(P1, Step::PrecombatMain);
    for p in [P0, P1, P2, P3] {
        bear(&mut t, p);
        bear(&mut t, p);
    }
    t.lands(P1, "Swamp", 1);
    let blood = t.hand(P1, "Innocent Blood");
    t.cast(P1, blood).go();
    t.script.lock().unwrap().asked.clear();
    t.resolve();
    let order = decisions_of(&t, |d| matches!(d, Decision::ChooseEntities { .. }));
    assert_eq!(order.len(), 4);
    let (first, second) = order.split_at(2);
    assert!(first.contains(&P0) && first.contains(&P1), "{order:?}");
    assert!(second.contains(&P2) && second.contains(&P3), "{order:?}");
}

#[test]
fn the_active_team_draws_first() {
    cr!("805.6a");
    // Vision Skeins: "Each player draws two cards."
    let mut t = thg();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 2);
    let skeins = t.hand(P1, "Vision Skeins");
    t.cast(P1, skeins).go();
    let from = t.g.turn_events.len();
    t.resolve();
    let drawers: Vec<PlayerId> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Drew { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(drawers.len(), 8);
    assert!(drawers[..4].iter().all(|p| *p == P0 || *p == P1), "{drawers:?}");
    assert!(drawers[4..].iter().all(|p| *p == P2 || *p == P3), "{drawers:?}");
}

#[test]
fn the_active_team_puts_all_its_triggered_abilities_on_the_stack_first() {
    cr!("805.7");
    // Soul Warden: "Whenever another creature enters, you gain 1 life." One each for P0,
    // P1 (the active team) and P2.
    let mut t = thg();
    for p in [P0, P1, P2] {
        t.battlefield(p, "Soul Warden");
    }
    t.script.lock().unwrap().asked.clear();
    t.enter(P3, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 3);
    // The active team orders both of its abilities together (its primary player
    // deciding); the nonactive team's go on the stack after them, on top.
    let orders: Vec<(PlayerId, usize)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Order { items, .. } => Some((p, items.len())),
            _ => None,
        })
        .collect();
    assert_eq!(orders, vec![(P0, 2)]);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).controller, P2);
}

#[test]
fn extra_turns_and_skipped_turns_are_the_teams() {
    cr!("805.8");
    // Time Warp: "Target player takes an extra turn after this one." P0 targets their
    // teammate P1: the team takes the extra turn.
    let mut t = thg();
    t.lands(P0, "Island", 5);
    let warp = t.hand(P0, "Time Warp");
    t.cast(P0, warp).target(P1).go();
    t.resolve();
    to_turn_of(&mut t, P1);
    assert!(t.g.turn.extra);
    assert!(t.g.is_active_player(P0) && t.g.is_active_player(P1));
    assert!(!t.g.is_active_player(P2));
    // Lethal Vapors: "{0}: Destroy this enchantment. You skip your next turn." P1 skips
    // their next turn: the team skips it.
    let mut t = thg();
    let vapors = t.battlefield(P2, "Lethal Vapors");
    t.activate(P1, vapors, 0, &[]).unwrap();
    t.resolve();
    to_turn_of(&mut t, P2);
    to_turn_of(&mut t, P2);
    assert!(
        t.g.turn.previous_active.is_some_and(|p| t.g.player(p).team == 1),
        "the P0/P1 team's turn was skipped"
    );
    // A single effect making both players of a team skip their next untap step: the team
    // skips only one untap step.
    let mut t = thg();
    t.set_step(P2, Step::PrecombatMain);
    let land = t.battlefield(P0, "Forest");
    t.g.objects[land.0 as usize].tapped = true;
    apply(
        &mut t,
        P2,
        Effect::Skip {
            who: PlayerRef::EachPlayer,
            step: StepKind::Untap,
        },
        &[],
    );
    to_turn_of(&mut t, P0);
    assert!(t.obj_now(land).tapped, "the team's untap step was skipped");
    to_turn_of(&mut t, P2);
    to_turn_of(&mut t, P0);
    assert!(!t.obj_now(land).tapped, "only once");
}

#[test]
fn the_active_player_refers_to_one_active_player_chosen_by_the_controller() {
    cr!("805.9");
    // "At the beginning of each end step, the active player draws a card." P2 controls
    // it; during P0's team's turn P2 chooses which active player it refers to.
    let mut t = thg();
    let engine = custom_with(
        "Turn Engine",
        "Enchantment",
        None,
        vec![triggered(
            TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::Any,
            },
            Effect::Draw {
                who: PlayerRef::ActivePlayer,
                n: Value::c(1),
            },
        )],
    );
    bf(&mut t, P2, engine);
    let before = (t.hand_size(P0), t.hand_size(P1));
    t.answer_choose(P2, &[Entity::Player(P1)]);
    go_to(&mut t, Step::End);
    t.resolve_all();
    assert_eq!(
        last_entity_candidates(&t, P2),
        vec![Entity::Player(P0), Entity::Player(P1)]
    );
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (before.0, before.1 + 1));
}

#[test]
fn a_team_attacks_the_other_team_as_a_group() {
    cr!("805.10", "805.10a", "805.10b");
    let mut t = thg();
    let a0 = bear(&mut t, P0);
    let a1 = bear(&mut t, P1);
    let jace = t.battlefield(P3, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attacking_players, vec![P0, P1]);
    assert_eq!(c.defending_players, vec![P2, P3]);
    // One combined attack: each attacking creature attacks a defending player, a
    // planeswalker or a battle.
    declare(&mut t, &[(a0, Entity::Player(P2)), (a1, Entity::Object(jace))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attack_target(&t, a0), Some(Entity::Player(P2)));
    assert_eq!(attack_target(&t, a1), Some(Entity::Object(jace)));
    // The set of attackers must be legal as a whole: Silent Arbiter ("No more than one
    // creature can attack each combat") limits the whole team.
    let mut t = thg();
    t.battlefield(P2, "Silent Arbiter");
    let a0 = bear(&mut t, P0);
    let a1 = bear(&mut t, P1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a0, Entity::Player(P2)), (a1, Entity::Player(P3))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attacking(&t).len() < 2);
}

#[test]
fn the_attacking_player_for_a_blocker_is_the_controller_of_what_it_blocks() {
    cr!("805.10c");
    // Goblin Goon: "This creature can't block unless you control more creatures than
    // attacking player." P2 controls two creatures; P0 controls one, P1 three.
    let mut t = thg();
    let a0 = bear(&mut t, P0);
    let a1 = bear(&mut t, P1);
    bear(&mut t, P1);
    bear(&mut t, P1);
    let goon = t.battlefield(P2, "Goblin Goon");
    bear(&mut t, P2);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a0, Entity::Player(P2)), (a1, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    let options = mtg_engine::combat::block_options(&t.g, &[P2, P3]);
    let can_block: Vec<ObjectId> = options
        .iter()
        .find(|(b, _)| *b == goon)
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    assert!(can_block.contains(&a0), "P0 controls fewer creatures than P2");
    assert!(!can_block.contains(&a1), "P1 doesn't");
}

#[test]
fn the_defending_team_declares_one_combined_block() {
    cr!("805.10d");
    let mut t = thg();
    let a0 = bear(&mut t, P0);
    let b2 = bear(&mut t, P2);
    let b3 = bear(&mut t, P3);
    t.set_step(P0, Step::BeginningOfCombat);
    // P0's creature attacks P2; P3's creature can block it too.
    declare(&mut t, &[(a0, Entity::Player(P2))]);
    block(&mut t, P2, &[(b2, a0), (b3, a0)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(
        decisions_of(&t, |d| matches!(d, Decision::DeclareBlockers { .. })),
        vec![P2]
    );
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blockers.len(), 2);
}

#[test]
fn the_defending_player_for_an_attacker_is_the_player_it_attacks() {
    cr!("805.10e");
    // Goblin Goon: "This creature can't attack unless you control more creatures than
    // defending player." P2 controls no creatures, P3 controls three.
    let mut t = thg();
    let goon = t.battlefield(P0, "Goblin Goon");
    bear(&mut t, P0);
    for _ in 0..3 {
        bear(&mut t, P3);
    }
    let targets = targets_of(&attack_choices(&mut t, P0), goon);
    assert!(targets.contains(&Entity::Player(P2)));
    assert!(!targets.contains(&Entity::Player(P3)));
}

#[test]
fn the_active_team_assigns_combat_damage_first() {
    cr!("805.10f");
    let mut t = thg();
    let wide = || {
        custom_card(
            "Wide Guard",
            "Creature — Soldier",
            Some((2, 4)),
            "This creature can block an additional creature each combat.",
        )
    };
    let a0 = bear(&mut t, P0);
    let a1 = bear(&mut t, P1);
    let a1b = bear(&mut t, P1);
    let b2 = bear(&mut t, P2);
    let b3 = bear(&mut t, P3);
    let g3 = bf(&mut t, P3, wide());
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (a0, Entity::Player(P2)),
            (a1, Entity::Player(P3)),
            (a1b, Entity::Player(P3)),
        ],
    );
    // P1's first attacker is blocked by two creatures; P3's guard blocks two attackers.
    block(&mut t, P2, &[(b2, a1), (b3, a1), (g3, a0), (g3, a1b)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::EndOfCombat);
    let order = decisions_of(&t, |d| matches!(d, Decision::AssignCombatDamage { .. }));
    assert_eq!(order, vec![P1, P3]);
}
