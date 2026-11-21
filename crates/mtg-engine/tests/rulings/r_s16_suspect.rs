//! Rulings on suspect (CR 701.60): any number of creatures can be suspected at once.

use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::suspect_detain::is_suspected;
use mtg_engine::testing::*;
use mtg_engine::*;

fn suspected(t: &TestGame, id: ObjectId) -> bool {
    is_suspected(&t.g, t.g.current(id))
}

#[test]
fn any_number_of_creatures_can_be_suspected_at_once() {
    cr!("701.60a", "701.60c");
    ruling!(
        "Barbed Servitor",
        "There's no limit to the number of creatures that can be suspected simultaneously. Suspecting a new creature doesn't cause other creatures to stop being suspected."
    );
    supported("Barbed Servitor");
    supported("Case of the Stashed Skeleton");
    supported("Convenient Target");
    let mut t = TestGame::new(2);
    // Barbed Servitor: "When this creature enters, suspect it."
    let servitor = t.enter(P0, "Barbed Servitor");
    t.resolve_all();
    assert!(suspected(&t, servitor));
    // Case of the Stashed Skeleton: "When this Case enters, create a 2/1 black Skeleton
    // creature token and suspect it."
    t.enter(P0, "Case of the Stashed Skeleton");
    t.resolve_all();
    let skeleton = with_subtype(&t, P0, "Skeleton")[0];
    assert!(suspected(&t, skeleton));
    assert!(suspected(&t, servitor));
    // Convenient Target: "When this Aura enters, suspect enchanted creature." on an
    // opponent's creature.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let aura = t.hand(P0, "Convenient Target");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    for c in [servitor, skeleton, bears] {
        assert!(suspected(&t, c));
        assert!(t.obj_now(c).chars.has_keyword(KeywordKind::Menace));
    }
}
