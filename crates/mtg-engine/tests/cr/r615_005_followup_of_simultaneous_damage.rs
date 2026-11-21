//! CR 615.5: a prevention effect's additional instruction happens immediately after it
//! prevents damage, and "that many" in it is the damage that would have been dealt. Applied
//! to simultaneous damage events (two attackers' combat damage, CR 510.2), the effect's
//! instruction is carried out for all of that damage: Angel of Suffering ("If damage would
//! be dealt to you, prevent that damage and mill twice that many cards.") mills twice the
//! total.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn angel_of_suffering_mills_twice_the_damage_two_attackers_would_deal_at_once() {
    cr!("615.5", "615.1a", "510.2");
    supported("Angel of Suffering");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Suffering");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let library = t.library_size(P0);
    // Grizzly Bears (2) and Hill Giant (3) unblocked: 5 damage to P0 at once, all of it
    // prevented. (Two 2-power attackers would mill 8 whether "that many" sums the damage or
    // multiplies it; 2 and 3 tell them apart: 10, not 12.)
    t.attack(
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P0))],
        &[],
    );
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.graveyard_size(P0), 10);
    assert_eq!(t.library_size(P0), library - 10);
}
