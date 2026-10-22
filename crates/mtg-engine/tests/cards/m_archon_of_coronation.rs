//! Archon of Coronation (hand-written, `src/cards/archon_of_coronation.rs`): while its
//! controller is the monarch, damage doesn't cause them to lose life.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn damage_doesnt_cause_the_monarch_to_lose_life() {
    cr!("120.3a", "614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archon of Coronation");
    t.g.monarch = Some(P0);
    t.g.recompute();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn not_the_monarch_loses_life_normally() {
    cr!("120.3a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archon of Coronation");
    t.g.monarch = Some(P1);
    t.g.recompute();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn damage_still_has_its_other_effects() {
    cr!("120.3a", "702.15b");
    ruling!("Archon of Coronation", "if the source has lifelink, its controller still gains life");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archon of Coronation");
    t.g.monarch = Some(P0);
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(knight, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 22);
    // The combat damage still made the attacker's controller the monarch.
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
}
