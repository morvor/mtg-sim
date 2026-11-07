//! Zilortha, Strength Incarnate (hand-written, `src/cards/zilortha_strength_incarnate.rs`):
//! lethal damage dealt to creatures its controller controls is determined by their power.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn lethal_damage_is_checked_against_power() {
    cr!("704.5g");
    ruling!("Zilortha, Strength Incarnate", "use the power of your creatures rather than their toughness");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zilortha, Strength Incarnate");
    // 1/4: two damage is lethal by power but not by toughness.
    let mine = t.battlefield(P0, "Horned Turtle");
    let theirs = t.battlefield(P1, "Horned Turtle");
    t.lands(P0, "Mountain", 2);
    let s1 = t.hand(P0, "Shock");
    t.cast(P0, s1).target(mine).go();
    t.resolve();
    let s2 = t.hand(P0, "Shock");
    t.cast(P0, s2).target(theirs).go();
    t.resolve();
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn zero_power_creature_needs_at_least_one_damage() {
    cr!("704.5g");
    ruling!("Zilortha, Strength Incarnate", "A creature with 0 power isn't destroyed unless it has at least 1 damage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zilortha, Strength Incarnate");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.settle();
    assert!(t.on_battlefield(wall));
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Shock");
    t.cast(P0, s).target(wall).go();
    t.resolve();
    assert!(!t.on_battlefield(wall));
}

#[test]
fn trample_assigns_lethal_damage_by_the_blockers_power() {
    cr!("702.19b");
    ruling!("Zilortha, Strength Incarnate", "This includes being assigned trample damage");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zilortha, Strength Incarnate");
    t.battlefield(P1, "Zilortha, Strength Incarnate");
    let turtle = t.battlefield(P1, "Horned Turtle");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(z, Entity::Player(P1))], &[(turtle, z)]);
    // 1 damage is lethal for the 1/4 blocker; the other 6 trample over.
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(turtle));
}
