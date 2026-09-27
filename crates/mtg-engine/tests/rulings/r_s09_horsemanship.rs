//! Rulings batch S09 — horsemanship (CR 702.31): "This creature can't be blocked except by
//! creatures with horsemanship."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn horsemanship_doesnt_interact_with_flying_or_reach() {
    cr!("702.31b", "702.9b", "702.17b");
    ruling!(
        "Wu Scout",
        "Despite the similarities between horsemanship and flying, horsemanship doesn't interact with flying or reach."
    );
    supported("Wu Scout");
    supported("Lu Xun, Scholar General");
    // A flyer and a creature with reach can't block a creature with horsemanship.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Wu Scout");
    let angel = t.battlefield(P1, "Serra Angel");
    let spider = t.battlefield(P1, "Giant Spider");
    let lu_xun = t.battlefield(P1, "Lu Xun, Scholar General");
    attack_with(&mut t, &[(scout, Entity::Player(P1))]);
    assert!(!t.g.can_block(angel, scout));
    assert!(!t.g.can_block(spider, scout));
    assert!(t.g.can_block(lu_xun, scout));
    block_and_finish(&mut t, P1, &[(angel, scout), (spider, scout)]);
    assert_eq!(t.life(P1), 19);
    // A creature with horsemanship can't block a flyer (a creature with reach can).
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let lu_xun = t.battlefield(P1, "Lu Xun, Scholar General");
    let spider = t.battlefield(P1, "Giant Spider");
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(!t.g.can_block(lu_xun, angel));
    assert!(t.g.can_block(spider, angel));
    block_and_finish(&mut t, P1, &[(lu_xun, angel)]);
    assert_eq!(t.life(P1), 16);
}
