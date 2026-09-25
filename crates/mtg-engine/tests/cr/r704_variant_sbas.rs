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
use mtg_engine::variants;
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

// --- 704.6e: schemes ------------------------------------------------------------------

#[test]
fn a_non_ongoing_scheme_returns_to_the_bottom_once_its_ability_has_left_the_stack() {
    cr!("704.6e");
    supported("Roots of All Evil");
    let mut t = archenemy_game();
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![
            (*mtg_engine::card::card("Roots of All Evil")).clone(),
            oracle_card("Idle Scheme", "Scheme", "", None, ""),
        ],
    );
    keyword_action(&mut t, P0, KeywordAction::SetInMotion, 1);
    // Its "when you set this scheme in motion" ability is waiting to be put on the stack,
    // then is on the stack: the scheme stays face up.
    assert!(!t.g.pending_triggers.is_empty());
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(!t.obj(deck[0]).face_down);
    assert_eq!(variants::face_up_schemes(&t.g), vec![deck[0]]);
    // Once it has resolved, the scheme is turned face down and put on the bottom of its
    // owner's scheme deck.
    t.resolve();
    assert!(t.obj(deck[0]).face_down);
    assert_eq!(variants::scheme_deck(&t.g, P0), vec![deck[1], deck[0]]);
    assert!(variants::face_up_schemes(&t.g).is_empty());
}

#[test]
fn a_scheme_waits_for_the_triggered_abilities_of_any_scheme() {
    cr!("704.6e");
    let mut t = archenemy_game();
    let ongoing = oracle_card(
        "Endless Plot",
        "Ongoing Scheme",
        "",
        None,
        "When you set this scheme in motion, you gain 2 life.\nAt the beginning of your end step, abandon this scheme.",
    );
    let idle = oracle_card("Idle Scheme", "Scheme", "", None, "");
    let deck = add_scheme_deck(&mut t, P0, vec![ongoing, idle]);
    // Set two schemes in motion, one at a time (CR 701.32c): the idle scheme has no
    // abilities of its own, but the ongoing scheme's ability hasn't left the stack yet.
    keyword_action(&mut t, P0, KeywordAction::SetInMotion, 2);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(!t.obj(deck[1]).face_down, "idle scheme waits");
    t.resolve();
    assert_eq!(t.life(P0), 42);
    assert!(t.obj(deck[1]).face_down);
    // An ongoing scheme stays face up...
    assert_eq!(variants::face_up_schemes(&t.g), vec![deck[0]]);
    t.settle();
    assert!(!t.obj(deck[0]).face_down);
    // ...until it's abandoned (CR 701.33b).
    to_step_start(&mut t, P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.obj(deck[0]).face_down);
    assert_eq!(variants::scheme_deck(&t.g, P0), vec![deck[1], deck[0]]);
}

// --- 704.6f: phenomena ----------------------------------------------------------------

#[test]
fn the_planar_controller_planeswalks_away_from_a_phenomenon_after_its_ability() {
    cr!("704.6f");
    use crate::r107_planechase::{add_planar_deck, face_up_names, planechase_game, roll};
    use mtg_engine::planechase::{self, PlanarFace};
    supported("Mutual Epiphany");
    let mut t = planechase_game(2, false);
    add_planar_deck(
        &mut t,
        P0,
        &["Goldmeadow", "Mutual Epiphany", "The Fourth Sphere"],
    );
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Planeswalker);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    t.resolve();
    // P0 planeswalked to the phenomenon: "When you encounter Mutual Epiphany, each player
    // draws four cards." While that ability is on the stack, the phenomenon stays.
    assert_eq!(face_up_names(&t), vec!["Mutual Epiphany"]);
    assert_eq!(t.stack_len(), 1);
    t.settle();
    assert_eq!(face_up_names(&t), vec!["Mutual Epiphany"]);
    // Once it has left the stack, the planar controller planeswalks.
    t.resolve();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1)),
        (hands.0 + 4, hands.1 + 4)
    );
    assert_eq!(face_up_names(&t), vec!["The Fourth Sphere"]);
    assert_eq!(t.stack_len(), 0);
}
