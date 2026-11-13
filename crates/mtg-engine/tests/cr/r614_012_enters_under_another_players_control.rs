//! CR 614.12: whether a replacement effect applies to a permanent entering the battlefield
//! is checked against the permanent as it would exist there, including the player who
//! will control it, not the card's owner.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_card_reanimated_from_an_opponents_graveyard_enters_tapped_under_your_control() {
    cr!("614.12", "614.1c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // "Creatures your opponents control enter tapped."
    t.battlefield(P1, "Authority of the Consuls");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    // "Put target creature card from a graveyard onto the battlefield under your control."
    let reanimate = t.hand(P0, "Reanimate");
    t.cast(P0, reanimate).target(Entity::Object(bears)).go();
    t.resolve_all();
    let now = t.obj_now(bears);
    assert_eq!(now.controller, P0);
    assert!(now.tapped, "it enters under P1's opponent's control");
}
