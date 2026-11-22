//! Rulings batch P127 — finality counters (CR 122.1i): a replacement effect of the
//! counter itself, not an ability; only for the graveyard; on any permanent; redundant
//! when several.

use crate::r_p127_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Rite of the Moth returns a Grizzly Bears with a finality counter on it.
fn returned_with_finality() -> (TestGame, ObjectId) {
    supported("Rite of the Moth");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Rite of the Moth", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(finality(&t, bears), 1);
    (t, bears)
}

#[test]
fn finality_counter_works_without_abilities() {
    cr!("122.1i", "613.1f");
    ruling!(
        "Rite of the Moth",
        "Finality counters aren't keyword counters, and a finality counter doesn't give any abilities to the permanent it's on. If that permanent loses its abilities and then would go to a graveyard, it will still be exiled instead."
    );
    let (mut t, bears) = returned_with_finality();
    assert!(t.obj_now(bears).chars.abilities.is_empty());
    cast_new(&mut t, P1, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    destroy(&mut t, bears);
    assert!(is_in(&t, bears, Zone::Exile));
}

#[test]
fn finality_counter_doesnt_stop_other_zone_changes() {
    cr!("122.1i");
    ruling!(
        "Rite of the Moth",
        "Finality counters don't stop permanents from going to zones other than the graveyard from the battlefield."
    );
    let (mut t, bears) = returned_with_finality();
    bounce(&mut t, bears);
    assert!(is_in(&t, bears, Zone::Hand(P0)));
}

#[test]
fn finality_counter_works_on_noncreature_permanents() {
    cr!("122.1i");
    ruling!(
        "Rite of the Moth",
        "Finality counters work on any permanent, not only creatures."
    );
    let mut t = TestGame::new(2);
    for name in ["Glorious Anthem", "Ornithopter", "Forest"] {
        let p = t.battlefield(P0, name);
        put_counters(&mut t, p, "finality", 1);
        destroy(&mut t, p);
        assert!(is_in(&t, p, Zone::Exile), "{name}");
    }
}

#[test]
fn multiple_finality_counters_are_redundant() {
    cr!("122.1i");
    ruling!(
        "Rite of the Moth",
        "Multiple finality counters on a single permanent are redundant."
    );
    let (mut t, bears) = returned_with_finality();
    put_counters(&mut t, bears, "finality", 1);
    assert_eq!(finality(&t, bears), 2);
    destroy(&mut t, bears);
    assert!(is_in(&t, bears, Zone::Exile));
    assert_eq!(t.g.exile.len(), 1);
}
