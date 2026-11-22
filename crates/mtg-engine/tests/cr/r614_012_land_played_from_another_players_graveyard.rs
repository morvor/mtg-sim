//! A replacement effect checked against the entering object where it is, before it moves
//! (Uphill Battle's "Creatures played by your opponents enter tapped.", whose "played" is
//! about how the card gets onto the battlefield), sees it with the controller it will have
//! there (CR 614.12, 110.2), not as its owner's card: a Dryad Arbor that P0 plays from
//! P1's graveyard (Shaman's Trance with Crucible of Worlds) is a creature one of P1's
//! opponents played, so it enters tapped under P1's Uphill Battle.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn uphill_battle_taps_its_controllers_dryad_arbor_an_opponent_plays_from_their_graveyard() {
    cr!("614.12", "614.1d", "110.2");
    ruling!("Dryad Arbor", "Dryad Arbor is played as a land.");
    supported("Uphill Battle");
    supported("Crucible of Worlds");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Uphill Battle");
    t.battlefield(P0, "Crucible of Worlds");
    let arbor = t.graveyard(P1, "Dryad Arbor");
    // "You may play lands and cast spells from other players' graveyards this turn as
    // though those cards were in your graveyard."
    cast_resolve(&mut t, P0, "Shaman's Trance", &[]);
    t.play_land(P0, arbor)
        .expect("Shaman's Trance and Crucible of Worlds let P0 play it");
    let a = t.g.current(arbor);
    assert!(t.on_battlefield(a));
    assert_eq!(t.obj_now(a).owner, P1);
    assert_eq!(t.obj_now(a).controller, P0);
    assert!(
        t.obj_now(a).tapped,
        "a creature P1's opponent played: checked under P0's control, not as P1's card"
    );
}
