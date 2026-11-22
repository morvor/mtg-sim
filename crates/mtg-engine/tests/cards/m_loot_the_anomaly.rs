//! Loot, the Anomaly (hand-written, `src/cards/loot_the_anomaly.rs`): with negative power
//! he assigns combat damage as though his power were positive (CR 510.1a).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn negative_power_assigns_its_absolute_value() {
    cr!("510.1a");
    let mut t = TestGame::new(2);
    let loot = t.battlefield(P0, "Loot, the Anomaly");
    assert_eq!(t.pt(loot), (-2, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(loot, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn blocked_he_assigns_that_damage_to_the_blocker() {
    cr!("510.1a", "510.1c");
    let mut t = TestGame::new(2);
    let loot = t.battlefield(P0, "Loot, the Anomaly");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(loot, Entity::Player(P1))], &[(bears, loot)]);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(loot));
}
