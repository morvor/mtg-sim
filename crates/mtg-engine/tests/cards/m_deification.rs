//! Deification (hand-written loyalty replacement, `src/cards/deification.rs`; its type
//! choice and hexproof compile).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn deification_compiles() {
    let def = mtg_engine::card::card("Deification");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
}

/// Deification with Jace chosen, P0's Jace Beleren, and a creature for P0 if `creature`.
fn setup(creature: bool) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let jace_type = mtg_engine::types::planeswalker_types()
        .iter()
        .position(|x| x == "Jace")
        .expect("Jace");
    t.answer(P0, DecisionKind::Option, Answer::Index(jace_type));
    t.enter(P0, "Deification");
    if creature {
        t.battlefield(P0, "Grizzly Bears");
    }
    let jace = t.battlefield(P0, "Jace Beleren");
    let giant = t.battlefield(P1, "Hill Giant");
    (t, jace, giant)
}

#[test]
fn damage_to_a_planeswalker_of_the_chosen_type_leaves_its_last_loyalty_counter() {
    cr!("120.3c", "614.1", "205.3j");
    ruling!(
        "Deification",
        "Deification's last ability doesn't prevent damage."
    );
    let (mut t, jace, giant) = setup(true);
    t.g.recompute();
    assert!(t.obj_now(jace).has_keyword(KeywordKind::Hexproof));
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    t.g.deal_damage(giant, Entity::Object(jace), 5, false);
    t.settle();
    assert!(t.on_battlefield(jace));
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    assert!(t.g.history.objects_dealt_damage.contains(&jace));
    // A planeswalker of another type: all of them are removed.
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let n = t.counters(chandra, counters::LOYALTY);
    t.g.deal_damage(giant, Entity::Object(chandra), n, false);
    t.settle();
    assert!(!t.on_battlefield(chandra));
    // Damage that leaves counters on it isn't replaced.
    let (mut t, jace, giant) = setup(true);
    t.g.deal_damage(giant, Entity::Object(jace), 2, false);
    t.settle();
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    // Without a creature: all of them are removed.
    let (mut t, jace, giant) = setup(false);
    t.g.deal_damage(giant, Entity::Object(jace), 5, false);
    t.settle();
    assert!(!t.on_battlefield(jace));
}
