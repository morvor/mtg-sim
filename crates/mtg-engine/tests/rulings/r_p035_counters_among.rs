//! Rulings batch P035 — "if there are N or more counters among [permanents]" (Lux
//! Artillery) and the other cards that condition (or the number "thirty") compiles for.

use crate::r_p035_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::can_attack;
use crate::r_s05_common::tokens_with_subtype;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn counters_among_cards_compile() {
    for n in [
        "Lux Artillery",
        "Backstreet Bruiser",
        "Knuckles the Echidna",
    ] {
        supported(n);
    }
}

#[test]
fn lux_artillery_checks_thirty_counters_twice() {
    cr!("603.4");
    ruling!(
        "Lux Artillery",
        "Lux Artillery's last ability will check if there are thirty or more counters among artifacts and creatures you control as your end step begins."
    );
    // Thirty counters among an artifact and a creature: 10 damage to each opponent.
    let mut t = TestGame::new(3);
    let lux = t.battlefield(P0, "Lux Artillery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    put(&mut t, lux, "charge", 10);
    put(&mut t, bears, counters::PLUS1, 20);
    put(&mut t, theirs, counters::PLUS1, 5);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.life(P2), 10);
    assert_eq!(t.life(P0), 20);
    // Twenty-nine (the opponent's counters don't count): no trigger.
    let mut t = TestGame::new(2);
    let lux = t.battlefield(P0, "Lux Artillery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    put(&mut t, lux, "charge", 9);
    put(&mut t, bears, counters::PLUS1, 20);
    put(&mut t, theirs, counters::PLUS1, 5);
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // Below thirty again before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let lux = t.battlefield(P0, "Lux Artillery");
    put(&mut t, lux, "charge", 30);
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    let l = t.g.current(lux);
    t.g.remove_counters(Entity::Object(l), "charge", 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn backstreet_bruiser_attacks_with_two_counters_around() {
    cr!("702.3b", "611.3a");
    let mut t = TestGame::new(2);
    let bruiser = t.battlefield(P0, "Backstreet Bruiser");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    put(&mut t, bears, counters::PLUS1, 1);
    assert!(!can_attack(&mut t, bruiser), "one counter");
    put(&mut t, bruiser, "charge", 1);
    assert!(
        can_attack(&mut t, bruiser),
        "two counters among creatures you control"
    );
}

#[test]
fn knuckles_treasures_and_thirty_artifacts() {
    cr!("702.4b", "104.2b");
    let mut t = TestGame::new(2);
    let knuckles = t.battlefield(P0, "Knuckles the Echidna");
    attack_with(&mut t, &[(knuckles, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "double strike");
    // A Treasure for each combat damage step in which it dealt damage.
    assert_eq!(tokens_with_subtype(&t, P0, "Treasure").len(), 2);
    for _ in 0..28 {
        t.battlefield(P0, "Sol Ring");
    }
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P1), "thirty artifacts");
}
