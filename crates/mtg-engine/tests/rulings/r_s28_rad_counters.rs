//! Rulings batch S28 — rad counters (CR 728) with Nightkin Ambusher ("When this creature
//! enters, target player gets four rad counters. This creature can't be blocked as long
//! as defending player has a rad counter."): counters a player has, the inherent
//! triggered ability at the beginning of that player's precombat main phase.

use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s11_common::empty_library;
use crate::r_s28_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn rad(t: &TestGame, p: PlayerId) -> u32 {
    player_counters(t, p, "rad")
}

/// Nightkin Ambusher enters for P0, and its trigger gives P1 four rad counters.
fn ambush(t: &mut TestGame) -> ObjectId {
    supported("Nightkin Ambusher");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ambusher = t.enter(P0, "Nightkin Ambusher");
    t.resolve_all();
    assert_eq!(rad(t, P1), 4);
    ambusher
}

#[test]
fn rad_counters_stay_until_an_effect_removes_them() {
    cr!("122.1", "728.1");
    ruling!(
        "Nightkin Ambusher",
        "Rad counters don't go away as steps, phases, or turns end. They only go away when an effect instructs a player to remove rad counters from themselves."
    );
    let mut t = TestGame::new(2);
    ambush(&mut t);
    // After P1's draw, three lands and a creature are on top: the rad ability mills
    // four cards and removes one counter.
    stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Forest", "Forest", "Forest", "Hill Giant"],
    );
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(rad(&t, P1), 4);
    t.advance_to(P1, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert_eq!(rad(&t, P1), 3);
    // Through the rest of the turn and the next one.
    t.advance_to(P0, Step::End);
    assert_eq!(rad(&t, P1), 3);
}

#[test]
fn rad_counters_are_a_players_not_a_permanents() {
    cr!("122.1", "728.1");
    ruling!(
        "Nightkin Ambusher",
        "Rad counters are a kind of counter that a player may have. They're not associated with any specific permanents."
    );
    let mut t = TestGame::new(2);
    let ambusher = ambush(&mut t);
    assert_eq!(t.counters(ambusher, "rad"), 0);
    crate::r_s02_common::destroy(&mut t, ambusher);
    assert!(t.in_graveyard(P0, "Nightkin Ambusher"));
    assert_eq!(rad(&t, P1), 4);
}

#[test]
fn the_rad_ability_has_no_source_and_is_the_active_players() {
    cr!("728.1");
    ruling!(
        "Nightkin Ambusher",
        "There is an inherent triggered ability associated with having rad counters. This triggered ability has no source and is controlled by the active player. The full text of this ability is \"At the beginning of the precombat main phase of a player with rad counters, that player mills cards equal to the number of rad counters they have. For each nonland card milled this way, that player loses 1 life and removes one rad counter from themselves.\""
    );
    let mut t = TestGame::new(2);
    ambush(&mut t);
    stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Forest", "Hill Giant", "Forest", "Hill Giant"],
    );
    t.advance_to(P1, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let ability = t.g.stack[0];
    assert_eq!(t.g.obj(ability).controller, P1);
    assert!(t.g.permanents().all(|o| o.id != ability));
    t.resolve_all();
    // Four milled, two nonland: 2 life, 2 counters.
    assert_eq!(t.graveyard_size(P1), 4);
    assert_eq!(t.life(P1), 18);
    assert_eq!(rad(&t, P1), 2);
}

#[test]
fn the_rad_ability_mills_as_many_as_it_can() {
    cr!("728.1", "701.17b");
    ruling!(
        "Nightkin Ambusher",
        "If a player has fewer cards remaining in their library than the number of rad counters they have when the triggered ability resolves, they'll mill as many cards as they can."
    );
    let mut t = TestGame::new(2);
    ambush(&mut t);
    // One card left after P1's draw.
    empty_library(&mut t, P1);
    stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    t.advance_to(P1, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 19);
    assert_eq!(rad(&t, P1), 3);
    assert!(!t.has_lost(P1));
}

#[test]
fn nightkin_ambusher_cant_be_blocked_while_the_defender_has_a_rad_counter() {
    cr!("509.1b", "728.1");
    let mut t = TestGame::new(2);
    let ambusher = ambush(&mut t);
    t.g.objects[ambusher.0 as usize].summoning_sick = false;
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(ambusher, Entity::Player(P1))]);
    assert!(crate::r_s10_common::attacking(&t, ambusher));
    assert!(!crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(bears, ambusher)]
    ));
    // Without rad counters it can be blocked.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let ambusher = t.enter(P0, "Nightkin Ambusher");
    t.resolve_all();
    t.g.objects[ambusher.0 as usize].summoning_sick = false;
    assert_eq!(rad(&t, P1), 0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(ambusher, Entity::Player(P1))]);
    assert!(crate::r_s10_common::attacking(&t, ambusher));
    assert!(crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(bears, ambusher)]
    ));
}

#[test]
fn a_blocked_nightkin_ambusher_stays_blocked_when_the_defender_gets_rad_counters() {
    cr!("509.1h");
    ruling!(
        "Nightkin Ambusher",
        "Once Nightkin Ambusher has been blocked, giving the defending player a rad counter won't cause Nightkin Ambusher to become unblocked."
    );
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let ambusher = t.enter(P0, "Nightkin Ambusher");
    t.resolve_all();
    t.g.objects[ambusher.0 as usize].summoning_sick = false;
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(ambusher, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(bears, ambusher)]),
    );
    crate::r_s21_common::go_to(&mut t, Step::DeclareBlockers);
    t.g.add_counters(Entity::Player(P1), "rad", 1, None);
    t.g.recompute();
    crate::r_s21_common::go_to(&mut t, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
}
