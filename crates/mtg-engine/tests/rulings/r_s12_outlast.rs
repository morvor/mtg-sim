//! Rulings batch S12 — outlast (CR 702.107): "[Cost], {T}: Put a +1/+1 counter on this
//! creature. Activate only as a sorcery."

use crate::r_s01_common::*;
use crate::r_s04_common::activate_named;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn outlast_taps_so_it_needs_the_creature_since_the_turn_began() {
    cr!("702.107a", "302.6", "602.5a");
    ruling!(
        "Longshot Squad",
        "The cost to activate a creature’s outlast ability includes the tap symbol ({T}). A creature’s outlast ability can’t be activated unless that creature has been under your control continuously since the beginning of your turn."
    );
    supported("Longshot Squad");
    // Longshot Squad: outlast {1}{G}; "Each creature you control with a +1/+1 counter on
    // it has reach."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    // One that just came under P0's control can't activate it.
    let new = t.battlefield_sick(P0, "Longshot Squad");
    assert!(activate_named(&mut t, P0, new, "Outlast", 0).is_err());
    assert_eq!(t.counters(new, counters::PLUS1), 0);
    // One controlled since the turn began can: it taps.
    let squad = t.battlefield(P0, "Longshot Squad");
    activate_named(&mut t, P0, squad, "Outlast", 0).unwrap();
    assert!(t.obj(squad).tapped);
    t.resolve_all();
    assert_eq!(t.counters(squad, counters::PLUS1), 1);
    assert!(t.obj(squad).has_keyword(KeywordKind::Reach));
    // Tapped, it can't be activated again.
    assert!(activate_named(&mut t, P0, squad, "Outlast", 0).is_err());
    assert_eq!(t.counters(squad, counters::PLUS1), 1);
}
