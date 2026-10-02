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
fn a_creature_with_negative_power_normally_deals_none() {
    cr!("510.1a");
    let mut t = TestGame::new(2);
    let loot = t.battlefield(P0, "Loot, the Anomaly");
    // Without the ability (a 0/x creature given -2/-0) no damage is assigned.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[loot.0 as usize].tapped = true;
    t.lands(P0, "Swamp", 1);
    let _ = bears;
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}
