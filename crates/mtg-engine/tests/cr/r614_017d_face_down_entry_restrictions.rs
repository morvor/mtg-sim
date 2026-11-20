//! CR 614.17d, 708.3: a card put onto the battlefield face down is turned face down before
//! it enters, so "[cards] in [zones] can't enter the battlefield" is checked against the
//! face-down card, with the characteristics the effect putting it there lists for it
//! (708.2a). Yedora, Grave Gardener ("Whenever another nontoken creature you control dies,
//! you may return it to the battlefield face down under its owner's control. It's a Forest
//! land.") returns a creature card from a graveyard as a face-down Forest land card: no
//! creature card (Grafdigger's Cage: "Creature cards in graveyards and libraries can't enter
//! the battlefield.") nor nonland permanent card (Weathered Runestone), but a permanent card
//! (Soulless Jailer: "Permanent cards in graveyards can't enter the battlefield."). A
//! manifested card is face down a 2/2 creature card, which Grafdigger's Cage stops
//! (`grafdiggers_cage_makes_manifesting_from_a_library_impossible`).

use crate::r614_audit_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn yedoras_face_down_forest_is_checked_as_a_forest_land_card() {
    cr!("614.17d", "708.3", "708.2a");
    ruling!(
        "Yedora, Grave Gardener",
        "The face-down card has no name or colors. Its only type is land, its only subtype is Forest"
    );
    ruling!(
        "Soulless Jailer",
        "Putting a permanent card onto the battlefield from a graveyard is an impossible action while Soulless Jailer is on the battlefield."
    );
    supported("Yedora, Grave Gardener");
    for (blocker, enters) in [
        ("Grafdigger's Cage", true),
        ("Weathered Runestone", true),
        ("Soulless Jailer", false),
    ] {
        supported(blocker);
        let mut t = TestGame::new(2);
        t.battlefield(P1, blocker);
        t.battlefield(P0, "Yedora, Grave Gardener");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.g.destroy(bears, None);
        t.resolve_all();
        let card = t.g.current(bears);
        if enters {
            assert!(t.on_battlefield(card), "{blocker}");
            let o = t.obj_now(card);
            assert!(o.face_down, "{blocker}");
            assert!(o.is(CardType::Land), "{blocker}");
            assert!(!o.is_creature(), "{blocker}");
        } else {
            assert_eq!(t.zone(card), Zone::Graveyard(P0), "{blocker}");
            assert!(!t.obj_now(card).face_down, "{blocker}");
        }
    }
}
