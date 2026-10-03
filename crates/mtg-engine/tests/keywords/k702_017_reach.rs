//! CR 702.17 Reach.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn reach_creatures_can_block_creatures_with_flying() {
    cr!("702.17", "702.17a", "702.17b", "702.9b");
    ruling!(
        "Giant Spider",
        "a creature with flying can only be blocked by creatures with flying or reach"
    );
    assert_supported("Giant Spider");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let spider = t.battlefield(P1, "Giant Spider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bird = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(t.g.can_block(spider, angel));
    assert!(t.g.can_block(bird, angel));
    assert!(!t.g.can_block(bears, angel));
    block_and_finish(&mut t, P1, &[(spider, angel)]);
    // The 4/4 Angel was blocked by the 2/4 Spider: no damage to the player.
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(spider));
    assert_eq!(t.obj_now(angel).damage, 2);
}

#[test]
fn reach_is_a_static_ability_not_flying() {
    cr!("702.17a", "702.17b");
    ruling!(
        "Treetop Rangers",
        "Creatures with reach (such as Giant Spider) don't actually have flying, so they can't block this."
    );
    ruling!(
        "Stone Spirit",
        "Creatures with the Reach ability (such as Giant Spider) can block this because they don't actually have Flying."
    );
    assert_supported("Treetop Rangers");
    assert_supported("Stone Spirit");
    // "This creature can't be blocked except by creatures with flying."
    let mut t = TestGame::new(2);
    let rangers = t.battlefield(P0, "Treetop Rangers");
    let spider = t.battlefield(P1, "Giant Spider");
    let bird = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(rangers, Entity::Player(P1))]);
    assert!(!t.g.can_block(spider, rangers));
    assert!(t.g.can_block(bird, rangers));
    // Elven Riders: "This creature can't be blocked except by Walls and/or creatures with
    // flying."
    ruling!(
        "Elven Riders",
        "Creatures with reach (such as Giant Spider) don't actually have flying, so they can't block this."
    );
    assert_supported("Elven Riders");
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Elven Riders");
    let spider = t.battlefield(P1, "Giant Spider");
    let bird = t.battlefield(P1, "Ornithopter");
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(riders, Entity::Player(P1))]);
    assert!(!t.g.can_block(spider, riders));
    assert!(t.g.can_block(bird, riders));
    assert!(t.g.can_block(wall, riders));
    assert!(!t.g.can_block(bears, riders));
    // "This creature can't be blocked by creatures with flying."
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Stone Spirit");
    let spider = t.battlefield(P1, "Giant Spider");
    let bird = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(spirit, Entity::Player(P1))]);
    assert!(t.g.can_block(spider, spirit));
    assert!(!t.g.can_block(bird, spirit));
    // Reach doesn't make an attacking creature harder to block.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Giant Spider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(spider, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, spider));
    // Granting reach works like having it.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let net = t.battlefield(P1, "Spidersilk Net");
    t.g.attach(net, Entity::Object(bears));
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Reach));
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, angel));
}

#[test]
fn multiple_instances_of_reach_are_redundant() {
    cr!("702.17c");
    assert_supported("Spidersilk Net");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let spider = t.battlefield(P1, "Giant Spider");
    let net = t.battlefield(P1, "Spidersilk Net");
    t.g.attach(net, Entity::Object(spider));
    t.g.recompute();
    assert_eq!(keyword_count(&t, spider, KeywordKind::Reach), 2);
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(t.g.can_block(spider, angel));
    // It still blocks only one creature.
    assert_eq!(t.g.max_blocks(spider), Some(1));
}
