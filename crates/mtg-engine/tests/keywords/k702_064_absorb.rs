//! CR 702.64 Absorb.

use crate::common_k702_011_017::{assert_supported, keyword_count};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn absorb_prevents_that_much_of_the_damage_a_source_would_deal() {
    cr!("702.64", "702.64a");
    assert_supported("Lymph Sliver");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P1, "Lymph Sliver");
    assert_eq!(keyword_count(&t, sliver, KeywordKind::Absorb), 1);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(sliver).go();
    t.resolve();
    assert!(t.on_battlefield(sliver));
    assert_eq!(t.obj_now(sliver).damage, 2);
    // It prevents only damage dealt to the creature itself.
    let pyromancer = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyromancer, 0, &[Entity::Object(sliver)])
        .unwrap();
    t.resolve();
    assert_eq!(t.obj_now(sliver).damage, 2);
}

#[test]
fn absorb_applies_to_each_source_and_to_each_time_damage_is_dealt() {
    cr!("702.64b");
    ruling!(
        "Lymph Sliver",
        "If multiple sources would deal damage to a Sliver at once, prevent 1 damage from each of those sources."
    );
    let mut t = TestGame::new(2);
    // Blocked by two 2/2s: 1 of each one's damage is prevented.
    let sliver = t.battlefield(P0, "Lymph Sliver");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(sliver, Entity::Player(P1))], &[(a, sliver), (b, sliver)]);
    assert!(t.on_battlefield(sliver));
    assert_eq!(t.obj_now(sliver).damage, 2);

    // A double striker's first-strike and regular damage are dealt at different times:
    // absorb prevents 1 of each.
    assert_supported("Mirran Crusader");
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P0, "Mirran Crusader");
    let sliver = t.battlefield(P1, "Lymph Sliver");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(crusader, Entity::Player(P1))], &[(sliver, crusader)]);
    assert!(t.on_battlefield(sliver));
    assert_eq!(t.obj_now(sliver).damage, 2);
}

#[test]
fn each_instance_of_absorb_applies_separately() {
    cr!("702.64c");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P1, "Lymph Sliver");
    t.battlefield(P1, "Lymph Sliver");
    // Each Lymph Sliver gives each Sliver absorb 1.
    assert_eq!(keyword_count(&t, first, KeywordKind::Absorb), 2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(first).go();
    t.resolve();
    assert_eq!(t.obj_now(first).damage, 1);
}
