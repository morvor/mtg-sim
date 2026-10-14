//! CR 810: the Two-Headed Giant variant.

use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 and P1 against P2 and P3.
fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

fn life_events(t: &TestGame, from: usize) -> Vec<Event> {
    t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::LifeLost { .. } | Event::LifeGained { .. }))
        .cloned()
        .collect()
}

#[test]
fn two_teams_of_two_players() {
    cr!("810.1");
    assert_eq!(GameConfig::two_headed_giant(vec![0, 0, 1, 1]).validate(4), Ok(()));
    for bad in [vec![0, 0, 1, 1, 2, 2], vec![0, 0, 0, 1]] {
        let n = bad.len();
        assert!(GameConfig::two_headed_giant(bad)
            .validate(n)
            .unwrap_err()
            .contains(&SetupError::TeamSizes));
    }
}

#[test]
fn two_headed_giant_uses_shared_team_turns() {
    cr!("810.2");
    let mut t = thg();
    assert!(t.g.uses_shared_team_turns());
    assert_eq!(t.g.active_players(), vec![P0, P1]);
    to_turn_of(&mut t, P2);
    assert_eq!(t.g.active_players(), vec![P2, P3]);
}

#[test]
fn each_team_sits_together() {
    cr!("810.3");
    // The participants' teams are 0, 1, 0, 1: each team sits together, in the order its
    // players chose.
    let decks: Vec<_> = (0..4).map(|_| super::r100_common::fillers(10)).collect();
    let g = mtg_engine::multiplayer::setup::new_seated(
        GameConfig::two_headed_giant(vec![0, 1, 0, 1]),
        decks,
        vec![],
    );
    assert_eq!(g.multiplayer.seats, vec![0, 2, 1, 3]);
    assert_eq!(g.config.teams, Some(vec![0, 0, 1, 1]));
    assert!(g.uses_shared_team_turns());
    // Teams that don't sit together aren't a valid Two-Headed Giant setup.
    assert!(GameConfig::two_headed_giant(vec![0, 1, 0, 1])
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::TeamsNotTogether));
}

#[test]
fn each_team_has_a_shared_life_total_starting_at_30() {
    cr!("810.4");
    let mut t = thg();
    for p in [P0, P1, P2, P3] {
        assert_eq!(t.life(p), 30);
    }
    t.g.lose_life(P3, 4);
    assert_eq!((t.life(P2), t.life(P3)), (26, 26));
    assert_eq!((t.life(P0), t.life(P1)), (30, 30));
}

#[test]
fn only_life_and_poison_counters_are_shared() {
    cr!("810.5");
    let mut t = thg();
    // P1 can't cast a spell from P0's hand, nor use P0's land for mana.
    let bolt = t.hand(P0, "Lightning Bolt");
    let land = t.battlefield(P0, "Mountain");
    t.g.turn.priority = Some(P1);
    let actions = t.g.legal_actions(P1);
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bolt)));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == land)));
    // Teammates may review each other's hands; opponents may not.
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, bolt));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P2, bolt));
}

#[test]
fn the_team_who_plays_first_skips_its_first_draw_step() {
    cr!("810.6");
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P2),
            skip_mulligans: true,
            ..GameConfig::two_headed_giant(vec![0, 0, 1, 1])
        },
        (0..4).map(|_| super::r100_common::fillers(40)).collect(),
    );
    t.g.start();
    to_step(&mut t, P2, Step::PrecombatMain);
    assert!(!t.g.turn.step_log.contains(&Step::Draw));
    assert_eq!((t.hand_size(P2), t.hand_size(P3)), (7, 7));
    to_step(&mut t, P0, Step::PrecombatMain);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (8, 8));
}

#[test]
fn two_headed_giant_uses_the_shared_team_turns_combat_rules() {
    cr!("810.7");
    let mut t = thg();
    let a0 = bear(&mut t, P0);
    let a1 = bear(&mut t, P1);
    let b3 = t.battlefield(P3, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    // P0's and P1's creatures attack together; P3's creature blocks one attacking P2.
    declare(&mut t, &[(a0, Entity::Player(P2)), (a1, Entity::Player(P2))]);
    block(&mut t, P2, &[(b3, a0)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!on_bf(&t, a0), "blocked and destroyed by P3's creature");
    assert_eq!(t.life(P2), 28);
    assert_eq!(t.life(P3), 28);
}

#[test]
fn teams_win_and_lose_together() {
    cr!("810.8", "810.8a");
    // If either player on a team wins, the team wins: Felidar Sovereign.
    let mut t = thg();
    t.battlefield(P0, "Felidar Sovereign");
    t.g.gain_life(P1, 10);
    assert_eq!(t.life(P0), 40);
    t.set_step(P3, Step::End);
    to_step(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0, P1])));
    // If either player on a team loses, the team loses.
    let mut t = thg();
    t.g.players[3].library.clear();
    t.g.draw_cards(P3, 1);
    t.settle();
    assert!(t.has_lost(P2) && t.has_lost(P3));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0, P1])));
    // "You can't lose the game" (Platinum Angel, P3) applies to the whole team.
    let mut t = thg();
    t.battlefield(P3, "Platinum Angel");
    t.g.lose_life(P2, 30);
    t.settle();
    assert!(!t.has_lost(P2) && !t.has_lost(P3));
    assert_eq!(t.g.result, None);
}

#[test]
fn a_player_who_concedes_takes_their_team_out() {
    cr!("810.8b");
    let mut t = thg();
    concede(&mut t, P1);
    assert!(t.has_lost(P0) && t.has_lost(P1));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P2, P3])));
}

#[test]
fn a_team_at_zero_life_loses() {
    cr!("810.8c");
    let mut t = thg();
    t.g.lose_life(P0, 29);
    t.settle();
    assert_eq!(t.g.result, None, "one life left");
    t.g.lose_life(P1, 1);
    t.settle();
    assert!(t.has_lost(P0) && t.has_lost(P1));
}

#[test]
fn a_team_with_fifteen_poison_counters_loses() {
    cr!("810.8d");
    let mut t = thg();
    t.g.add_counters(Entity::Player(P2), counters::POISON, 8, None);
    t.g.add_counters(Entity::Player(P3), counters::POISON, 6, None);
    t.settle();
    assert_eq!(t.g.result, None, "14 poison counters");
    t.g.add_counters(Entity::Player(P3), counters::POISON, 1, None);
    t.settle();
    assert!(t.has_lost(P2) && t.has_lost(P3));
}

#[test]
fn damage_and_life_changes_happen_to_players_and_apply_to_the_team() {
    cr!("810.9");
    let mut t = thg();
    let from = t.g.turn_events.len();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P2).go();
    t.resolve();
    // P2 was dealt the damage and lost the life; the team's total went down.
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 1);
    assert!(matches!(evs[0], Event::LifeLost { player, amount: 3 } if player == P2));
    assert_eq!((t.life(P2), t.life(P3)), (27, 27));
}

#[test]
fn an_individual_life_total_is_the_teams() {
    cr!("810.9a");
    ruling!(
        "Serra Ascendant",
        "Serra Ascendant gets +5/+5 and has flying as long as your team has 30 or more life"
    );
    let mut t = thg();
    let asc = t.battlefield(P0, "Serra Ascendant");
    t.g.recompute();
    assert_eq!(t.pt(asc), (6, 6));
    // P1 loses life: the team's (and so P0's) life total drops below 30.
    t.g.lose_life(P1, 1);
    t.g.recompute();
    assert_eq!(t.pt(asc), (1, 1));
}

#[test]
fn teammates_paying_life_together_cant_exceed_the_team_life_total() {
    cr!("810.9b");
    // "Each player draws a card unless they pay 2 life." With 3 life, P0 and P1 can't both
    // pay 2 life.
    let mut t = thg();
    t.g.lose_life(P0, 27);
    let each = Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::PayOptional {
            who: PlayerRef::Iterated,
            cost: Cost::free().with(CostPart::PayLife(Value::c(2))),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Draw {
                who: PlayerRef::Iterated,
                n: Value::c(1),
            }),
        }),
    };
    for p in [P0, P1, P2, P3] {
        t.answer_yes(p, true);
    }
    let hands = (t.hand_size(P0), t.hand_size(P1));
    apply(&mut t, P2, each, &[]);
    assert_eq!(t.life(P0), 1, "only one of them paid");
    assert_eq!(t.hand_size(P0) + t.hand_size(P1), hands.0 + hands.1 + 1);
    assert_eq!(t.life(P2), 26, "P2 and P3 paid 2 each");
}

#[test]
fn setting_one_players_life_total_adjusts_the_team() {
    cr!("810.9c");
    ruling!(
        "Magister Sphinx",
        "this ability basically causes the team's life total to become 10, but only the targeted player is considered to have actually gained or lost life"
    );
    let mut t = thg();
    t.answer_targets(P0, &[Entity::Player(P2)]);
    let from = t.g.turn_events.len();
    t.enter(P0, "Magister Sphinx");
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (10, 10));
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 1);
    assert!(matches!(evs[0], Event::LifeLost { player, amount: 20 } if player == P2));
}

#[test]
fn setting_each_players_life_total_affects_one_chosen_member_per_team() {
    cr!("810.9d");
    // "Each player's life total becomes 10." Each team chooses one of its members; only
    // that player's life total becomes 10.
    let mut t = thg();
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P2, &[Entity::Player(P3)]);
    let from = t.g.turn_events.len();
    apply(
        &mut t,
        P0,
        Effect::SetLife {
            who: PlayerRef::EachPlayer,
            n: Value::c(10),
        },
        &[],
    );
    assert_eq!((t.life(P0), t.life(P2)), (10, 10));
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 2);
    assert!(evs
        .iter()
        .all(|e| matches!(e, Event::LifeLost { player, amount: 20 } if *player == P1 || *player == P3)));
}

#[test]
fn teammates_cant_exchange_life_totals() {
    cr!("810.9e");
    ruling!("Soul Conduit", "If the two targeted players are teammates, nothing will happen.");
    let mut t = thg();
    t.g.lose_life(P2, 10);
    let conduit = t.battlefield(P0, "Soul Conduit");
    t.lands(P0, "Wastes", 6);
    t.activate(P0, conduit, 0, &[Entity::Player(P0), Entity::Player(P1)])
        .unwrap();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P2)), (30, 20));
    // Players on different teams do: the teams exchange life totals.
    let conduit2 = t.battlefield(P0, "Soul Conduit");
    t.lands(P0, "Wastes", 6);
    t.activate(P0, conduit2, 0, &[Entity::Player(P1), Entity::Player(P3)])
        .unwrap();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P2)), (20, 30));
}

#[test]
fn redistributing_life_totals_affects_at_most_one_player_per_team() {
    cr!("810.9f");
    // Reverse the Sands: "Redistribute any number of players' life totals."
    let mut t = thg();
    t.g.lose_life(P2, 25);
    t.lands(P0, "Plains", 8);
    let sands = t.hand(P0, "Reverse the Sands");
    // P0 tries to redistribute P0's, P1's and P2's life totals: P1 is P0's teammate, so
    // only P0 and P2 are affected. P0 takes P2's 5 and P2 gets P0's 30.
    t.answer_choose(
        P0,
        &[Entity::Player(P0), Entity::Player(P1), Entity::Player(P2)],
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, sands).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (5, 5));
    assert_eq!((t.life(P2), t.life(P3)), (30, 30));
}

#[test]
fn a_player_who_cant_gain_life_stops_the_team_gaining_life() {
    cr!("810.9g");
    // Platinum Emperion (P0): "Your life total can't change."
    let mut t = thg();
    t.battlefield(P0, "Platinum Emperion");
    t.lands(P1, "Plains", 3);
    t.set_step(P1, Step::PrecombatMain);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[0]).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 30);
    assert_eq!(t.g.gain_life(P1, 5), 0);
    assert_eq!(t.g.gain_life(P2, 5), 5, "the other team isn't affected");
}

#[test]
fn a_player_who_cant_lose_life_stops_the_team_losing_or_paying_life() {
    cr!("810.9h");
    let mut t = thg();
    t.battlefield(P0, "Platinum Emperion");
    t.lands(P2, "Mountain", 1);
    let bolt = t.hand(P2, "Lightning Bolt");
    t.cast(P2, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 30);
    assert!(!t.g.can_pay_life(P1, 1));
    assert!(t.g.can_pay_life(P1, 0));
}

#[test]
fn players_get_poison_counters_individually_and_the_team_shares_them() {
    cr!("810.10");
    // Glistener Elf has infect: its combat damage to P2 gives P2 poison counters.
    let mut t = thg();
    let elf = t.battlefield(P0, "Glistener Elf");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(elf, Entity::Player(P2))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.g.player(P2).poison(), 1);
    assert_eq!(t.g.player(P3).poison(), 0);
    assert_eq!(
        mtg_engine::multiplayer::two_headed::player_counter(&t.g, P3, counters::POISON),
        1
    );
}

#[test]
fn a_players_poison_counters_are_their_teams() {
    cr!("810.10a");
    // "This deals damage to target player equal to the number of poison counters that
    // player has."
    let mut t = thg();
    t.g.add_counters(Entity::Player(P2), counters::POISON, 3, None);
    let source = bear(&mut t, P0);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), P0);
    ctx.targets = vec![vec![Entity::Player(P3)]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            to: Sel::Target(0),
            amount: Value::PlayerCounters(PlayerRef::Target(0), counters::POISON.into()),
        },
        &mut ctx,
    );
    assert_eq!(t.life(P3), 27);
}

#[test]
fn a_player_losing_poison_counters_means_the_team_loses_them() {
    cr!("810.10b");
    let mut t = thg();
    t.g.add_counters(Entity::Player(P2), counters::POISON, 4, None);
    // P3 loses two poison counters: the team does, from P2.
    assert_eq!(t.g.remove_counters(Entity::Player(P3), counters::POISON, 2), 2);
    assert_eq!(t.g.player(P2).poison(), 2);
}

#[test]
fn a_player_who_cant_get_poison_counters_protects_the_team() {
    cr!("810.10c");
    let mut t = thg();
    bf(
        &mut t,
        P2,
        custom_card(
            "Antidote Idol",
            "Artifact",
            None,
            "You can't get poison counters.",
        ),
    );
    assert_eq!(t.g.add_counters(Entity::Player(P3), counters::POISON, 2, None), 0);
    assert_eq!(t.g.add_counters(Entity::Player(P2), counters::POISON, 2, None), 0);
    assert_eq!(t.g.add_counters(Entity::Player(P0), counters::POISON, 2, None), 2);
}

#[test]
fn a_player_is_poisoned_if_their_team_has_poison_counters() {
    cr!("810.10d");
    let mut t = thg();
    t.g.add_counters(Entity::Player(P2), counters::POISON, 1, None);
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.player_filter_matches(&PlayerFilter::Poisoned, P3, &ctx));
    assert!(!t.g.player_filter_matches(&PlayerFilter::Poisoned, P1, &ctx));
    // P3 has the kinds of counters their team has: proliferate can give P3 poison.
    let kinds = mtg_engine::kwa::proliferate_teams::player_counter_kinds(&t.g, P3);
    assert!(kinds.iter().any(|k| k.as_str() == counters::POISON));
}

#[test]
fn larger_teams_have_more_life_and_need_more_poison() {
    cr!("810.11");
    let mut t = TestGame::with_config(
        6,
        GameConfig::two_headed_giant(vec![0, 0, 0, 1, 1, 1]),
    );
    assert_eq!(t.life(P0), 45);
    assert_eq!(GameConfig::two_headed_giant(vec![0, 0, 0, 1, 1, 1]).validate(6), Ok(()));
    t.g.add_counters(Entity::Player(P3), counters::POISON, 19, None);
    t.settle();
    assert_eq!(t.g.result, None, "Three-Headed Giant teams lose at 20");
    t.g.add_counters(Entity::Player(P4), counters::POISON, 1, None);
    t.settle();
    assert!(t.has_lost(P3) && t.has_lost(P4) && t.has_lost(P5));
}
