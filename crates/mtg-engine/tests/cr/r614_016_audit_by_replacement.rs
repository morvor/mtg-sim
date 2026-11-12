//! CR 614.16 audit: effects that modify "an effect" creating tokens or putting counters
//! also modify the tokens and counters another replacement or prevention effect creates or
//! puts, even when the event it replaced (a death by lethal damage, combat damage) wasn't
//! an effect's.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn doubling_season_doubles_the_zombies_kalitas_creates_for_creatures_killed_by_damage() {
    cr!("614.16", "704.5g");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalitas, Traitor of Ghet");
    t.battlefield(P0, "Doubling Season");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    // Pyroclasm's damage is lethal to the Bears (Kalitas, a 3/4, survives); the state-based
    // action destroys them, and Kalitas's replacement effect creates a Zombie for each,
    // which Doubling Season doubles.
    cast_resolve(&mut t, P0, "Pyroclasm", &[]);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
    assert_eq!(t.named_on_battlefield("Zombie Token").len(), 4);
    assert_eq!(t.named_on_battlefield("Kalitas, Traitor of Ghet").len(), 1);
}

/// P1's Hill Giant attacks P0, whose Grizzly Bears block; P0 controls Vigor and `extra`.
fn giant_blocked_by_bears_under_vigor(extra: &str) -> (TestGame, ObjectId) {
    supported("Vigor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vigor");
    t.battlefield(P0, extra);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(giant, Entity::Player(P0))], &[(bears, giant)]);
    (t, bears)
}

#[test]
fn hardened_scales_adds_to_the_counters_vigor_puts_for_prevented_combat_damage() {
    cr!("614.16", "615.5");
    let (t, bears) = giant_blocked_by_bears_under_vigor("Hardened Scales");
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 0);
    // 3 prevented, so three counters, plus one.
    assert_eq!(plus_counters(&t, bears), 4);
}

#[test]
fn doubling_season_doubles_the_counters_vigor_puts_for_prevented_combat_damage() {
    cr!("614.16", "615.5");
    let (t, bears) = giant_blocked_by_bears_under_vigor("Doubling Season");
    assert!(t.on_battlefield(bears));
    // "If an effect would put one or more counters": Vigor's prevention effect puts them.
    assert_eq!(plus_counters(&t, bears), 6);
}
