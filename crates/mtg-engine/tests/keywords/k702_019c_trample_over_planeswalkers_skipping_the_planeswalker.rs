//! CR 702.19c: a creature with trample over planeswalkers attacking a planeswalker can
//! assign damage to that planeswalker's controller only once the planeswalker is assigned
//! lethal damage, so an assignment of nothing to the planeswalker and the rest to its
//! controller is illegal (the default assignment is used instead).

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn trample_over_planeswalkers_cant_skip_the_planeswalker() {
    cr!("702.19c");
    let mut t = TestGame::new(2);
    // 7/7 trample over planeswalkers (and trample).
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    assert_eq!(t.counters(jace, "loyalty"), 3);
    let giant = t.battlefield(P1, "Hill Giant");
    // 3 to the blocker, none to Jace, 4 to P1: illegal, so the default (3, 3, 1) is used.
    assign_damage(&mut t, P0, &[3, 0, 4]);
    attack_with(&mut t, &[(thrasta, Entity::Object(jace))]);
    block_and_finish(&mut t, P1, &[(giant, thrasta)]);
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 19);
}
