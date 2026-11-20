//! Item `user-decisions-nov20`, part 4: a creature that Swift Reconfiguration made a
//! noncreature Vehicle artifact loses its creature types with its creature card type
//! (CR 205.1a); crewed later (a later layer-4 effect that makes it an artifact creature,
//! CR 702.122a, 205.1b, 613.3), it is a creature again but has no creature types
//! (interpretation `subtypes-removed-with-a-card-type-stay-removed-once-it-returns`, decided
//! by the user Nov 20).

use crate::r_residue_type_change_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn grizzly_bears_under_swift_reconfiguration_crewed_has_no_creature_types() {
    cr!("205.1a", "205.1b", "613.1d", "702.122a");
    interp!("subtypes-removed-with-a-card-type-stay-removed-once-it-returns");
    supported("Swift Reconfiguration");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giants = [
        t.battlefield(P0, "Hill Giant"),
        t.battlefield(P0, "Hill Giant"),
    ];
    assert_eq!(creature_types(&mut t, bears), vec!["Bear".to_string()]);
    // "Enchanted permanent is a Vehicle artifact with crew 5 and it loses all other card
    // types. (It's not a creature unless it's crewed.)"
    cast_new(&mut t, P0, "Swift Reconfiguration", &[obj(bears)]);
    t.resolve_all();
    assert!(!is_type(&mut t, bears, CardType::Creature));
    assert!(is_type(&mut t, bears, CardType::Artifact));
    assert!(subtype(&mut t, bears, "Vehicle"));
    assert!(creature_types(&mut t, bears).is_empty(), "removed with the type");
    // Crewed by the two Hill Giants: an artifact creature Vehicle with no creature types.
    assert!(crate::r_s04_common::crew(&mut t, P0, bears, &giants));
    t.resolve_all();
    assert!(is_type(&mut t, bears, CardType::Creature), "crewed");
    assert!(is_type(&mut t, bears, CardType::Artifact));
    assert!(subtype(&mut t, bears, "Vehicle"));
    assert!(
        creature_types(&mut t, bears).is_empty(),
        "no creature types once crewed: {:?}",
        creature_types(&mut t, bears)
    );
    assert!(!subtype(&mut t, bears, "Bear"));
    assert_eq!(pt_now(&mut t, bears), (2, 2));
}
