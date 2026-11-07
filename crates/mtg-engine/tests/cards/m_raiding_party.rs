//! Raiding Party (hand-written, `src/cards/raiding_party.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn plains_not_saved_by_white_creatures_are_destroyed() {
    cr!("701.8a");
    ruling!("Raiding Party", "The creatures are tapped during spell resolution");
    let mut t = TestGame::new(2);
    let rp = t.battlefield(P0, "Raiding Party");
    let orc = t.battlefield(P0, "Orcish Artillery");
    let lions = t.battlefield(P1, "Savannah Lions");
    let plains = t.lands(P1, "Plains", 3);
    t.answer_choose(P1, &[Entity::Object(lions)]);
    t.answer_choose(P1, &[Entity::Object(plains[0]), Entity::Object(plains[1])]);
    t.answer_choose(P0, &[Entity::Object(orc)]);
    t.activate(P0, rp, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(lions).tapped);
    assert!(t.on_battlefield(plains[0]));
    assert!(t.on_battlefield(plains[1]));
    assert!(!t.on_battlefield(plains[2]));
}
