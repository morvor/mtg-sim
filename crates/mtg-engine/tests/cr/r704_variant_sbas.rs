//! CR 704.6: state-based actions of variant games (and the variant-only state-based
//! actions of 704.5).

use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Variant;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn two_headed() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            starting_life: 40,
            ..Default::default()
        },
    )
}

#[test]
fn a_two_headed_giant_team_with_no_life_loses() {
    cr!("704.6", "704.6a");
    let mut t = two_headed();
    // Damage to one player is applied to the team's shared life total.
    t.g.players[2].life = 3;
    t.g.players[3].life = 3;
    t.g.lose_life(P3, 3);
    t.settle();
    assert!(t.has_lost(P2) && t.has_lost(P3));
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
}

#[test]
fn a_two_headed_giant_team_with_fifteen_poison_counters_loses() {
    cr!("704.5c", "704.6b");
    let mut t = two_headed();
    // Ten poison counters on one player doesn't make that player lose in Two-Headed
    // Giant (704.5c is ignored): the team's poison counters count.
    t.g.add_counters(Entity::Player(P2), counters::POISON, 10, None);
    t.settle();
    assert!(!t.has_lost(P2));
    // The team shares its poison counters: five more on the teammate make fifteen.
    t.g.add_counters(Entity::Player(P3), counters::POISON, 5, None);
    t.settle();
    assert!(t.has_lost(P2) && t.has_lost(P3));
    assert!(!t.has_lost(P0));
}

#[test]
fn twenty_one_combat_damage_from_one_commander_loses_the_game() {
    cr!("704.6c");
    let mut t = commander_game();
    let cmdr = t.battlefield(P0, "Hill Giant");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[1]
        .commander_damage
        .insert("Hill Giant".into(), 18);
    t.settle();
    assert!(!t.has_lost(P1));
    // Three more combat damage from the same commander makes 21.
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(cmdr, Entity::Player(P1))]),
    );
    t.g.run_until(1000, |g| {
        g.result.is_some() || g.turn.step == Step::EndOfCombat
    });
    assert_eq!(t.player(P1).commander_damage.get("Hill Giant"), Some(&21));
    assert!(t.has_lost(P1));
    assert!(t.life(P1) > 0);
}

#[test]
fn a_commander_put_into_a_graveyard_or_exile_may_go_to_the_command_zone() {
    cr!("704.6d");
    let mut t = commander_game();
    let cmdr = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.objects[giant.0 as usize].is_commander = true;
    // The owner chooses: yes for the Bears, no for the Giant.
    t.answer_yes(P0, true);
    t.g.destroy(cmdr, None);
    t.settle();
    let in_command = t.g.find_in_zone(Zone::Command, "Grizzly Bears");
    assert_eq!(in_command.len(), 1);
    assert!(t.g.obj(in_command[0]).is_commander);
    t.answer_yes(P0, false);
    t.g.exile_object(giant, None);
    t.settle();
    assert!(t.in_exile("Hill Giant"));
    // Only since the last check: declining once, it stays in exile.
    let asked = t.asked().len();
    t.settle();
    t.g.add_counters(Entity::Player(P1), counters::POISON, 1, None);
    t.settle();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.asked()[asked..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::YesNo { .. })));
}
