//! "Damage that would reduce your life total to less than 1 reduces it to 1 instead"
//! (Worship) looks at the whole damage event, including the life its player gains as part
//! of it (CR 120.4d): lifelink damage dealt at the same time counts (CR 120.3f, 702.15b).
//! P1 at 3 life is dealt 6 combat damage while P1's lifelink blocker deals 2: the event
//! [lose 6 life, gain 2 life] would reduce 3 life to -1, so Worship applies and P1 ends at
//! 1. (Counting the gained life the other way, as 3 - 2 = 1 life, would make Worship
//! reduce the loss to 0, and P1 would end at 5.)

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn worship_counts_the_life_a_lifelink_blocker_gains_in_the_same_damage_event() {
    cr!("120.4d", "120.3f", "702.15b", "614.1a");
    ruling!(
        "Worship",
        "It reduces your life total to 1, not the damage to 1."
    );
    supported("Worship");
    supported("Child of Night");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Worship");
    let child = t.battlefield(P1, "Child of Night");
    t.g.players[P1.idx()].life = 3;
    // Colossal Dreadmaw (6/6) unblocked; Grizzly Bears blocked by Child of Night (2/1
    // lifelink), whose 2 damage to the Bears gains P1 2 life in the same event.
    let wurm = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(
        &[(wurm, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[(child, bears)],
    );
    // The loss is reduced to 4 (3 life plus the 2 gained, down to 1), then the 2 is gained.
    assert_eq!(t.life(P1), 1);
    assert!(!t.has_lost(P1));
}
