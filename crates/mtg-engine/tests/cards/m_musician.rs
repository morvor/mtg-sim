//! Musician (hand-written, `src/cards/musician.rs`): a creature gains "At the beginning
//! of your upkeep, destroy this creature unless you pay {1} for each music counter on it"
//! only if it doesn't already have it.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const GRANTED: &str =
    "At the beginning of your upkeep, destroy ~ unless you pay {1} for each music counter on it.";

fn granted(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text == GRANTED)
        .count()
}

#[test]
fn two_musicians_add_counters_but_grant_the_ability_once() {
    cr!("113.10");
    ruling!("Musician", "Although a creature can gain the triggered ability only once");
    let mut t = TestGame::new(2);
    let m1 = t.battlefield(P0, "Musician");
    let m2 = t.battlefield(P0, "Musician");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, m1, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "music"), 1);
    assert_eq!(granted(&t, bears), 1);
    t.activate(P0, m2, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "music"), 2);
    assert_eq!(granted(&t, bears), 1);
}

#[test]
fn the_creature_is_destroyed_unless_its_controller_pays_for_each_counter() {
    cr!("113.10");
    ruling!("Musician", "It doesn’t matter if Musician is still on the battlefield or not.");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Musician");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, m, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    // The Musician leaves; the ability stays.
    t.g.destroy(m, None);
    t.lands(P1, "Forest", 1);
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Draw);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn music_counters_alone_grant_nothing() {
    cr!("122.1");
    ruling!("Musician", "that creature won’t have the triggered ability");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "music", 2, None);
    t.advance_to(P1, Step::Draw);
    assert!(t.on_battlefield(bears));
    assert_eq!(granted(&t, bears), 0);
}
