//! Rulings on start your engines! and max speed (CR 702.178, 702.179): what a max speed
//! ability means, on the battlefield and in other zones.

use crate::r_s01_common::*;
use crate::r_s16_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_max_speed_ability_is_granted_only_at_max_speed_even_outside_the_battlefield() {
    cr!("702.178a", "702.178b", "702.179e");
    ruling!(
        "Muraganda Raceway",
        "“Max speed — [ability]” means “As long as you have max speed, this object has [ability].” If the granted ability functions in a zone other than the battlefield, the max speed ability does too."
    );
    supported("Muraganda Raceway");
    supported("Lightwheel Enhancements");
    // Muraganda Raceway: "{T}: Add {C}. Max speed — {T}: Add {C}{C}." It's P0's only mana
    // source, and Mind Stone costs {2}.
    let mut t = TestGame::new(2);
    let raceway = t.battlefield(P0, "Muraganda Raceway");
    let stone = t.hand(P0, "Mind Stone");
    set_speed(&mut t, P0, 3);
    assert!(t.cast(P0, stone).try_go().is_err());
    t.clear_answers();
    assert!(t.in_hand(P0, "Mind Stone"));
    assert!(!t.obj_now(raceway).tapped);
    // At max speed the Raceway has "{T}: Add {C}{C}".
    set_speed(&mut t, P0, 4);
    t.cast(P0, stone).go();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Mind Stone").is_empty());

    // Lightwheel Enhancements: "Max speed — You may cast this card from your graveyard."
    // The granted ability functions in the graveyard, so the max speed ability does too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.graveyard(P0, "Lightwheel Enhancements");
    t.lands(P0, "Plains", 1);
    set_speed(&mut t, P0, 3);
    assert!(t.cast(P0, aura).target(bears).try_go().is_err());
    t.clear_answers();
    assert!(t.in_graveyard(P0, "Lightwheel Enhancements"));
    set_speed(&mut t, P0, 4);
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}
