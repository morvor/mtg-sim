//! Fireball (hand-written, `src/cards/fireball.rs`): X damage divided evenly, rounded
//! down, among the targets still legal as it resolves.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn divides_evenly_rounding_down() {
    cr!("601.2f");
    ruling!("Fireball", "it deals 1 damage to each of them");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    // X = 5 among three targets: {5}{R} plus {2} for the two extra targets.
    t.lands(P0, "Mountain", 8);
    let f = t.hand(P0, "Fireball");
    t.cast(P0, f)
        .x(5)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Player(P1)])
        .go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 1);
}

#[test]
fn divides_among_the_targets_still_legal() {
    cr!("608.2b");
    ruling!("Fireball", "The division involves only targets that are still legal");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 8);
    let f = t.hand(P0, "Fireball");
    t.cast(P0, f)
        .x(6)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    // One target leaves before Fireball resolves: the other (a 6/4) is dealt all 6
    // rather than 3.
    t.g.destroy(b, None);
    t.resolve();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn more_targets_than_x_deals_nothing() {
    cr!("608.2b");
    ruling!("Fireball", "none of them will be dealt any damage");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let f = t.hand(P0, "Fireball");
    t.cast(P0, f)
        .x(1)
        .targets(&[Entity::Object(a), Entity::Player(P1)])
        .go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(a).damage, 0);
}
