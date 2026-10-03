//! Rulings batch S10 — infect (CR 702.90): damage to creatures in the form of -1/-1
//! counters and to players in the form of poison counters.

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::next_upkeep;
use crate::r_s10_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn ten_poison_counters_lose_the_game_as_a_state_based_action() {
    cr!("704.5c", "702.90b", "104.3d");
    ruling!(
        "Contagious Nim",
        "A player who has ten or more poison counters loses the game. This is a state-based action."
    );
    supported("Contagious Nim");
    // P1 has eight poison counters; Contagious Nim (2/2, infect) deals 2 combat damage.
    let mut t = TestGame::new(2);
    let nim = t.battlefield(P0, "Contagious Nim");
    t.g.add_counters(Entity::Player(P1), counters::POISON, 8, None);
    t.settle();
    assert!(!t.has_lost(P1));
    attack_with(&mut t, &[(nim, Entity::Player(P1))]);
    // No blocks; the game ends in the combat damage step.
    t.g.run_until(10_000, |g| g.result.is_some());
    assert_eq!(poison(&t, P1), 10);
    assert_eq!(t.life(P1), 20);
    assert!(t.has_lost(P1));
    // The player loses when state-based actions are next checked, not as the counters
    // are put on them.
    let mut t = TestGame::new(2);
    t.g.add_counters(Entity::Player(P1), counters::POISON, 11, None);
    assert!(!t.has_lost(P1));
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn prevented_infect_damage_gives_no_poison_or_minus_counters() {
    cr!("702.90b", "702.90c", "615.1");
    ruling!(
        "Contagious Nim",
        "If damage from a source with infect that would be dealt to a player is prevented, that player doesn’t get poison counters. If damage from a source with infect that would be dealt to a creature is prevented, that creature doesn’t get -1/-1 counters."
    );
    supported("Fog");
    let mut t = TestGame::new(2);
    let nim = t.battlefield(P0, "Contagious Nim");
    let other = t.battlefield(P0, "Contagious Nim");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(nim, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    // P1 casts Fog: "Prevent all combat damage that would be dealt this turn."
    t.lands(P1, "Forest", 1);
    let fog = t.hand(P1, "Fog");
    t.cast(P1, fog).go();
    t.resolve();
    block_and_finish(&mut t, P1, &[(bears, other)]);
    assert_eq!(poison(&t, P1), 0);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    assert!(t.on_battlefield(bears));
}

#[test]
fn infect_applies_to_noncombat_damage() {
    cr!("702.90b", "702.90c", "701.14a");
    ruling!(
        "Contagious Nim",
        "Infect’s effect applies to any damage, not just combat damage."
    );
    supported("Prey Upon");
    // Contagious Nim (2/2) fights Hill Giant (3/3): the Giant gets two -1/-1 counters
    // and no damage; the Nim is dealt 3 damage.
    let mut t = TestGame::new(2);
    let nim = t.battlefield(P0, "Contagious Nim");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey)
        .targets(&[Entity::Object(nim)])
        .targets(&[Entity::Object(giant)])
        .go();
    t.resolve();
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.pt(giant), (1, 1));
    assert!(t.in_graveyard(P0, "Contagious Nim"));
}

/// The infect creature `name` fights Cudgel Troll ({G}: Regenerate); the -1/-1 counters
/// stay on the Troll when it regenerates and when the turn ends.
fn counters_stay(name: &str, power: u32) {
    supported(name);
    supported("Cudgel Troll");
    let mut t = TestGame::new(2);
    let infect = t.battlefield(P0, name);
    let troll = t.battlefield(P1, "Cudgel Troll");
    t.lands(P0, "Forest", 1);
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey)
        .targets(&[Entity::Object(infect)])
        .targets(&[Entity::Object(troll)])
        .go();
    t.resolve();
    assert_eq!(t.counters(troll, counters::MINUS1), power);
    // Regenerate the Troll and destroy it: it's regenerated, the counters stay.
    t.lands(P1, "Forest", 1);
    t.activate(P1, troll, 0, &[]).expect("regenerate");
    t.resolve();
    destroy(&mut t, troll);
    assert!(t.on_battlefield(troll));
    assert!(t.obj_now(troll).tapped);
    assert_eq!(t.counters(troll, counters::MINUS1), power);
    // The turn ends: they're still there.
    next_upkeep(&mut t, P1);
    assert_eq!(t.counters(troll, counters::MINUS1), power);
    assert_eq!(t.pt(troll), (4 - power as i32, 3 - power as i32));
}

#[test]
fn minus_counters_from_infect_stay_through_regeneration_and_turn_end() {
    cr!("702.90c", "701.19a", "514.2");
    ruling!(
        "Contagious Nim",
        "The -1/-1 counters remain on the creature indefinitely. They’re not removed if the creature regenerates or the turn ends."
    );
    counters_stay("Contagious Nim", 2);
}

#[test]
fn minus_counters_from_infect_stay_through_regeneration_and_turn_end_straight() {
    cr!("702.90c", "701.19a", "514.2");
    ruling!(
        "Plague Stinger",
        "The -1/-1 counters remain on the creature indefinitely. They're not removed if the creature regenerates or the turn ends."
    );
    counters_stay("Plague Stinger", 1);
}
