//! Preparation cards: "enters prepared", "becomes prepared" (CR 722).

use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// The exiled prepare-spell copy of a prepared permanent.
fn prepared_copy(t: &TestGame, perm: ObjectId) -> Option<ObjectId> {
    t.obj_now(perm).prepared
}

#[test]
fn prepare_cards_compile() {
    for n in [
        "Adventurous Eater // Have a Bite",
        "Goblin Glasswright // Craft with Pride",
        "Infirmary Healer // Stream of Life",
    ] {
        assert_supported(n);
    }
}

#[test]
fn enters_prepared_creates_castable_copy_in_exile() {
    cr!("722.3a", "722.3c");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "this does not apply to copies of prepare spells in exile"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let eater = t.hand(P0, "Adventurous Eater // Have a Bite");
    t.cast(P0, eater).go();
    t.resolve();
    assert!(t.on_battlefield(eater));
    let copy = prepared_copy(&t, eater).expect("prepared");
    // The copy is in exile with only the prepare spell's characteristics, and it
    // survives state-based actions (CR 722.3c exception to 704.5e).
    t.settle();
    assert_eq!(t.g.obj(copy).zone, Zone::Exile);
    assert_eq!(t.g.obj(copy).kind, ObjKind::CardCopy);
    assert_eq!(t.g.obj(copy).chars.name.as_str(), "Have a Bite");
    assert!(t.g.permitted_cards(P0).contains(&copy));
    assert!(!t.g.permitted_cards(P1).contains(&copy));

    // Cast the copy: "Put a +1/+1 counter on target creature. You gain 1 life."
    let now = t.g.current(eater);
    let spell = t.cast(P0, copy).target(now).go();
    // The permanent loses the designation as the spell becomes cast (CR 722.3c).
    assert!(prepared_copy(&t, eater).is_none());
    t.resolve();
    assert_eq!(t.counters(eater, "+1/+1"), 1);
    assert_eq!(t.life(P0), 21);
    // The copy then ceases to exist (CR 704.5e).
    let after = t.g.current(spell);
    assert_eq!(t.g.obj(after).zone, Zone::Nowhere);
    assert!(!t.in_graveyard(P0, "Have a Bite"));
}

#[test]
fn prepare_spell_cant_be_cast_from_hand() {
    cr!("722.3");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "Preparation cards can only be cast with their base characteristics"
    );
    let mut t = TestGame::new(2);
    let eater = t.hand(P0, "Adventurous Eater // Have a Bite");
    let opts = t.g.cast_options(P0, eater);
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].face, object::FaceState::Front);
}

#[test]
fn copy_ceases_to_exist_when_permanent_leaves() {
    cr!("722.3c", "704.5e");
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, "Adventurous Eater // Have a Bite");
    let copy = prepared_copy(&t, eater).expect("prepared");
    t.lands(P1, "Mountain", 1);
    // Lightning Bolt kills the 3/2.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(eater).go();
    t.resolve();
    assert!(!t.on_battlefield(eater));
    assert_eq!(t.g.obj(copy).zone, Zone::Nowhere);
    assert!(!t.g.permitted_cards(P0).contains(&copy));
}

#[test]
fn only_current_controller_may_cast_the_copy() {
    cr!("722.3c");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "Only the current controller of a prepared creature can cast the copy"
    );
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, "Adventurous Eater // Have a Bite");
    let copy = prepared_copy(&t, eater).unwrap();
    let now = t.g.current(eater);
    t.g.objects[now.0 as usize].controller = P1;
    t.g.objects[now.0 as usize].base_controller = P1;
    assert!(t.g.permitted_cards(P1).contains(&copy));
    assert!(!t.g.permitted_cards(P0).contains(&copy));
}

#[test]
fn becomes_prepared_trigger_and_unprepared() {
    cr!("722.3a", "722.3b");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "If it wasn't prepared at that time, nothing happens"
    );
    // "Whenever this creature attacks, it becomes prepared."
    assert_supported("Encouraging Aviator // Jump");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Encouraging Aviator // Jump");
    assert!(prepared_copy(&t, a).is_none());
    t.set_step(P0, turn::Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    let copy = prepared_copy(&t, a).expect("became prepared");
    // Becoming prepared again while prepared does nothing (CR 722.3a).
    let now = t.g.current(a);
    designations::become_prepared(&mut t.g, now);
    assert_eq!(prepared_copy(&t, a), Some(copy));
    // Becoming unprepared removes the designation; the copy ceases to exist.
    designations::become_unprepared(&mut t.g, now);
    t.settle();
    assert!(prepared_copy(&t, a).is_none());
    assert_eq!(t.g.obj(copy).zone, Zone::Nowhere);
}

#[test]
fn creature_without_prepare_spell_cant_become_prepared() {
    cr!("722.3a");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "A creature without a prepare spell can't become prepared"
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    designations::become_prepared(&mut t.g, b);
    assert!(t.obj_now(b).prepared.is_none());
}
