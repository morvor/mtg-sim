//! Which replacement effects modify how a permanent enters is checked against the permanent
//! as it would exist on the battlefield, with "continuous effects that already exist and
//! would apply to the permanent" (CR 614.12). A creature entering as a copy of another one
//! still has the abilities such effects grant it, riot from Rhythm of the Wild ("Nontoken
//! creatures you control have riot.") among them.

use crate::r614_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Regression test: once a permanent was entering as a copy, `Game::replace_rec`'s candidates
/// took its own "enters" abilities from the copiable values only, not from the permanent as
/// it would exist (`Game::replacement_candidates` now takes them from its characteristics
/// in `with_hypothetical_entry`), so the riot Rhythm of the Wild grants it was lost: Clone
/// entering as a copy of Grizzly Bears got neither riot's counter nor haste (Grizzly Bears
/// entering itself gets riot, see `k702_136_riot.rs`).
#[test]
fn clone_entering_as_a_copy_under_rhythm_of_the_wild_has_riot() {
    cr!("614.12", "702.136a");
    supported("Rhythm of the Wild");
    supported("Clone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhythm of the Wild");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Clone: "You may have this creature enter as a copy of any creature on the
    // battlefield." Then riot: "+1/+1 counter?" yes.
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Grizzly Bears");
    assert!(
        plus_counters(&t, clone) == 1 || t.obj_now(clone).has_keyword(KeywordKind::Haste),
        "riot's counter or haste"
    );
}
