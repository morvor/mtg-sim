//! CR 702.2 Deathtouch on spells ("Instant and sorcery spells you control have
//! deathtouch").

use crate::common_k702_011_017::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn instant_and_sorcery_spells_you_control_have_deathtouch() {
    cr!("702.2b", "702.2d");
    ruling!(
        "Pestilent Spirit",
        "If the spell instructs another object to deal damage, the spell doesn't deal any damage itself and its instance of deathtouch doesn't apply."
    );
    assert_supported("Pestilent Spirit");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pestilent Spirit");
    // A spell dealing damage from the stack: any damage destroys the creature.
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // A spell that has a creature deal the damage doesn't.
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let bite = t.hand(P0, "Rabid Bite");
    t.cast(P0, bite).target(bears).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    // An opponent's spells don't have it.
    let own_giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(own_giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(own_giant));
}
