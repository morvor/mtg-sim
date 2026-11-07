//! "You may have [subject] [verb] ..." (pattern in
//! `src/oracle/patterns/damage_removal_have.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn have_cards_compile() {
    assert_compiles(&[
        "Affectionate Indrik",
        "Blood Seeker",
        "Death Pulse",
        "Extractor Demon",
        "Ob Nixilis, the Fallen",
    ]);
}

#[test]
fn affectionate_indrik_may_fight_a_creature_you_dont_control() {
    cr!("701.14a", "603.5");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    let indrik = t.enter(P0, "Affectionate Indrik");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(indrik).damage, 2);
}

#[test]
fn affectionate_indrik_fight_is_optional() {
    cr!("603.5");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, false);
    let indrik = t.enter(P0, "Affectionate Indrik");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(indrik).damage, 0);
}

#[test]
fn blood_seeker_has_that_player_lose_life() {
    cr!("603.5", "119.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Seeker");
    t.answer_yes(P0, true);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn death_pulse_cycling_trigger_shrinks_the_target() {
    cr!("702.29a", "603.5");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 3);
    let pulse = t.hand(P0, "Death Pulse");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_yes(P0, true);
    // Cycling is the card's only activated ability.
    t.activate(P0, pulse, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elves), "a 1/1 with -1/-1 dies");
}
