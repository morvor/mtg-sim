//! Rulings batch S15 — scavenge (CR 702.97): Dreg Mangler, with Tormod's Crypt.

use crate::r_s01_common::*;
use crate::r_s04_common::activate_named;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn removing_the_card_from_the_graveyard_after_scavenge_is_activated_is_too_late() {
    cr!("702.97a", "602.2b");
    ruling!(
        "Dreg Mangler",
        "Exiling the creature card with scavenge is part of the cost of activating the scavenge ability. Once the ability is activated and the cost is paid, it's too late to stop the ability from being activated by trying to remove the creature card from the graveyard."
    );
    supported("Dreg Mangler");
    supported("Tormod's Crypt");
    // Dreg Mangler (3/3): "Scavenge {3}{B}{G}".
    let mut t = TestGame::new(2);
    let mangler = t.graveyard(P0, "Dreg Mangler");
    let other = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, mangler, "Scavenge", 0).expect("scavenge");
    // The card was exiled to pay the cost.
    assert_eq!(t.zone(mangler), Zone::Exile);
    // In response, P1 exiles P0's graveyard with Tormod's Crypt ("{T}, Sacrifice Tormod's
    // Crypt: Exile all cards from target player's graveyard.").
    t.activate(P1, crypt, 0, &[Entity::Player(P0)]).unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(other), Zone::Exile);
    // The scavenge ability still resolves: three +1/+1 counters.
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    assert_eq!(t.pt(bears), (5, 5));
}
