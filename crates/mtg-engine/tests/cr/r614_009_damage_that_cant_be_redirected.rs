//! Damage that "can't be prevented or dealt instead to another permanent or player" (Lava
//! Burst's to a creature) isn't redirected (CR 614.9), but the replacement effects that
//! aren't redirections still apply to it: Furnace of Rath doubles it (CR 614.1a).

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn lava_burst_to_a_creature_isnt_redirected_but_is_still_doubled() {
    cr!("614.9", "614.1a");
    ruling!(
        "Lava Burst",
        "Prevention and redirection effects will have no effect on damage dealt by Lava Burst to a creature."
    );
    supported("Lava Burst");
    supported("Furnace of Rath");
    supported("Palisade Giant");
    // Furnace of Rath: "If a source would deal damage to a permanent or player, it deals
    // double that damage to that permanent or player instead." Palisade Giant: "All damage
    // that would be dealt to you and other permanents you control is dealt to this creature
    // instead." Lava Burst: "Lava Burst deals X damage to any target. If Lava Burst would deal
    // damage to a creature, that damage can't be prevented or dealt instead to another
    // permanent or player."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace of Rath");
    let giant = t.battlefield(P1, "Palisade Giant");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 3);
    let burst = t.hand(P0, "Lava Burst");
    t.cast(P0, burst).x(2).target(obj(dreadmaw)).go();
    t.resolve();
    // X = 2, doubled, all to Colossal Dreadmaw (6/6).
    assert_eq!(damage_on(&t, dreadmaw), 4);
    assert_eq!(damage_on(&t, giant), 0);
    assert!(t.on_battlefield(dreadmaw));
}
