//! CR 702.177 Exhaust: cards that modify, copy, or are exhaust abilities.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn boom_scholar_reduces_other_permanents_exhaust_costs() {
    cr!("702.177a", "602.2b");
    assert_supported("Boom Scholar");
    // Boom Scholar: "Exhaust abilities of other permanents you control cost {2} less to
    // activate." "Exhaust — {4}{R}{G}: ..." Elvish Refueler: "Exhaust — {1}{G}: Put a
    // +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let scholar = t.battlefield(P0, "Boom Scholar");
    let refueler = t.battlefield(P0, "Elvish Refueler");
    let theirs = t.battlefield(P1, "Elvish Refueler");
    let own = ability_uid(&mut t, scholar, "Exhaust");
    let other = ability_uid(&mut t, refueler, "Exhaust");
    // {G} is enough for the Refueler's {1}{G} (the generic part can't go below zero).
    add_mana(&mut t, P0, ManaType::G, 1);
    assert!(activatable(&mut t, P0, refueler, other));
    // Boom Scholar's own exhaust ability isn't reduced: {4}{R}{G} needs six mana.
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::G, 3);
    assert!(!activatable(&mut t, P0, scholar, own));
    // An opponent's permanent's exhaust ability isn't reduced either.
    let their_uid = ability_uid(&mut t, theirs, "Exhaust");
    t.g.players[P0.idx()].mana_pool = Default::default();
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P1, ManaType::G, 1);
    assert!(!activatable(&mut t, P1, theirs, their_uid));
    t.set_step(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::G, 1);
    activate_uid(&mut t, P0, refueler, other).unwrap();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.counters(refueler, "+1/+1"), 1);
}

#[test]
fn spire_mechcycle_counts_other_mounts_and_vehicles() {
    cr!("702.177a");
    assert_supported("Spire Mechcycle");
    // Spire Mechcycle: "Exhaust — Tap another untapped Mount or Vehicle you control: This
    // Vehicle becomes an artifact creature. Put a +1/+1 counter on it for each Mount
    // and/or Vehicle you control other than this Vehicle."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cycle = t.battlefield(P0, "Spire Mechcycle");
    let refueler = t.battlefield(P0, "Rangers' Refueler");
    t.battlefield(P0, "Brightfield Mustang");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Brightfield Mustang");
    let uid = ability_uid(&mut t, cycle, "Exhaust");
    t.answer_choose(P0, &[Entity::Object(refueler)]);
    activate_uid(&mut t, P0, cycle, uid).unwrap();
    assert!(t.obj(refueler).tapped);
    t.resolve_all();
    assert!(t.obj(cycle).chars.is(CardType::Creature));
    // The tapped Refueler and the Mustang: two counters.
    assert_eq!(t.counters(cycle, "+1/+1"), 2);
    assert_eq!(t.pt(cycle), (7, 6));
}

#[test]
fn riverchurn_monument_mills_any_number_of_target_players() {
    cr!("702.177a", "701.17a");
    assert_supported("Riverchurn Monument");
    // Riverchurn Monument: "{1}, {T}: Any number of target players each mill two cards."
    // "Exhaust — {2}{U}{U}, {T}: Any number of target players each mill cards equal to the
    // number of cards in their graveyard."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let monument = t.battlefield(P0, "Riverchurn Monument");
    for _ in 0..20 {
        t.library_top(P0, "Forest");
        t.library_top(P1, "Forest");
    }
    let mill = ability_uid(&mut t, monument, "{1}, {T}");
    add_mana(&mut t, P0, ManaType::U, 1);
    t.answer_targets(P0, &[Entity::Player(P0), Entity::Player(P1)]);
    activate_uid(&mut t, P0, monument, mill).unwrap();
    t.resolve_all();
    assert_eq!((t.graveyard_size(P0), t.graveyard_size(P1)), (2, 2));
    // Only the opponent: three more cards in their graveyard, then twice as many.
    t.graveyard(P1, "Forest");
    t.g.untap(monument);
    let exhaust = ability_uid(&mut t, monument, "Exhaust");
    add_mana(&mut t, P0, ManaType::U, 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_uid(&mut t, P0, monument, exhaust).unwrap();
    t.resolve_all();
    assert_eq!((t.graveyard_size(P0), t.graveyard_size(P1)), (2, 6));
}

#[test]
fn pit_automaton_copies_the_next_exhaust_ability() {
    cr!("702.177a", "707.10");
    assert_supported("Pit Automaton");
    // Pit Automaton: "{2}, {T}: When you next activate an exhaust ability that isn't a mana
    // ability this turn, copy it. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let automaton = t.battlefield(P0, "Pit Automaton");
    let refueler = t.battlefield(P0, "Elvish Refueler");
    let paragon = t.battlefield(P0, "Pacesetter Paragon");
    let uid = ability_uid(&mut t, automaton, "{2}, {T}");
    add_mana(&mut t, P0, ManaType::C, 2);
    activate_uid(&mut t, P0, automaton, uid).unwrap();
    t.resolve_all();
    // The Refueler's exhaust ability and its copy: two counters.
    let first = ability_uid(&mut t, refueler, "Exhaust");
    add_mana(&mut t, P0, ManaType::G, 2);
    activate_uid(&mut t, P0, refueler, first).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(refueler, "+1/+1"), 2);
    // Only the next one.
    let second = ability_uid(&mut t, paragon, "Exhaust");
    add_mana(&mut t, P0, ManaType::R, 3);
    activate_uid(&mut t, P0, paragon, second).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(paragon, "+1/+1"), 1);
}
