//! Rulings batch P221 — prepared (CR 722.3): effects that make a creature become prepared
//! or unprepared, and a prepared creature becoming prepared again after its copy is cast.

use crate::r_s01_common::*;
use crate::r_s29_common::choose_modes;
use mtg_engine::designations;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Emeritus of Conflict ({1}{R} 2/2 first strike, "Whenever you cast your third spell each
/// turn, this creature becomes prepared.") // Lightning Bolt.
const EMERITUS: &str = "Emeritus of Conflict // Lightning Bolt";

fn prepared_copy(t: &TestGame, perm: ObjectId) -> Option<ObjectId> {
    t.obj_now(perm).prepared
}

/// Activates Skycoach Waypoint's "{3}, {T}: Target creature becomes prepared." ability.
fn waypoint(t: &mut TestGame, target: ObjectId) {
    t.lands(P0, "Wastes", 3);
    let w = t.battlefield(P0, "Skycoach Waypoint");
    t.activate(P0, w, 1, &[Entity::Object(target)])
        .expect("Skycoach Waypoint");
    t.resolve_all();
}

#[test]
fn skycoach_waypoint_does_nothing_to_a_creature_without_a_prepare_spell_or_already_prepared() {
    cr!("722.3a", "722.3c");
    ruling!(
        "Skycoach Waypoint",
        "Skycoach Waypoint's last ability won't have any effect on a creature that has no prepare spell or is already prepared."
    );
    supported("Skycoach Waypoint");
    supported(EMERITUS);
    // A creature with no prepare spell doesn't become prepared.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    waypoint(&mut t, bears);
    assert!(prepared_copy(&t, bears).is_none());
    assert!(t.g.exile.is_empty());
    // An unprepared creature with a prepare spell does (the control case).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let emeritus = t.battlefield(P0, EMERITUS);
    waypoint(&mut t, emeritus);
    assert!(prepared_copy(&t, emeritus).is_some());
    // An already-prepared one: nothing happens (same copy, no second copy).
    let mut t2 = TestGame::new(2);
    t2.set_step(P0, Step::PrecombatMain);
    let emeritus = t2.battlefield(P0, EMERITUS);
    designations::become_prepared(&mut t2.g, emeritus);
    let copy2 = prepared_copy(&t2, emeritus).expect("prepared");
    waypoint(&mut t2, emeritus);
    assert_eq!(prepared_copy(&t2, emeritus), Some(copy2));
    assert_eq!(t2.g.exile.len(), 1);
}

#[test]
fn biblioplex_tomekeeper_modes_only_affect_creatures_they_can_change() {
    cr!("722.3a", "722.3b", "700.2");
    ruling!(
        "Biblioplex Tomekeeper",
        "The first mode of Biblioplex Tomekeeper's last ability won't have any effect on a creature that has no prepare spell or is already prepared. Similarly, the second mode won't have any effect on a creature that isn't already prepared."
    );
    supported("Biblioplex Tomekeeper");
    // Mode 1 ("becomes prepared") on a creature with no prepare spell: nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    choose_modes(&mut t, P0, &[0]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Biblioplex Tomekeeper");
    t.resolve_all();
    assert!(prepared_copy(&t, bears).is_none());
    assert!(t.g.exile.is_empty());
    // Mode 1 on an already-prepared creature: it keeps its one copy.
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, EMERITUS);
    designations::become_prepared(&mut t.g, emeritus);
    let copy = prepared_copy(&t, emeritus).unwrap();
    choose_modes(&mut t, P0, &[0]);
    t.answer_targets(P0, &[Entity::Object(emeritus)]);
    t.enter(P0, "Biblioplex Tomekeeper");
    t.resolve_all();
    assert_eq!(prepared_copy(&t, emeritus), Some(copy));
    assert_eq!(t.g.exile.len(), 1);
    // Mode 1 on an unprepared creature with a prepare spell: it becomes prepared.
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, EMERITUS);
    choose_modes(&mut t, P0, &[0]);
    t.answer_targets(P0, &[Entity::Object(emeritus)]);
    t.enter(P0, "Biblioplex Tomekeeper");
    t.resolve_all();
    assert!(prepared_copy(&t, emeritus).is_some());
    // Mode 2 ("becomes unprepared") on a creature that isn't prepared: nothing happens.
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, EMERITUS);
    choose_modes(&mut t, P0, &[1]);
    t.answer_targets(P0, &[Entity::Object(emeritus)]);
    t.enter(P0, "Biblioplex Tomekeeper");
    t.resolve_all();
    assert!(prepared_copy(&t, emeritus).is_none());
    assert!(t.g.exile.is_empty());
    // Mode 2 on a prepared creature: it becomes unprepared and its copy ceases to exist.
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, EMERITUS);
    designations::become_prepared(&mut t.g, emeritus);
    choose_modes(&mut t, P0, &[1]);
    t.answer_targets(P0, &[Entity::Object(emeritus)]);
    t.enter(P0, "Biblioplex Tomekeeper");
    t.resolve_all();
    assert!(prepared_copy(&t, emeritus).is_none());
    assert!(t.g.exile.is_empty());
}

#[test]
fn casting_the_prepared_copy_as_the_third_spell_prepares_it_again() {
    cr!("722.3a", "722.3c", "601.2i", "603.2");
    ruling!(
        "Emeritus of Conflict // Lightning Bolt",
        "casting that copy of Lightning Bolt as your third spell in a turn will result in Emeritus of Conflict becoming prepared again when its last ability resolves."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let emeritus = t.battlefield(P0, EMERITUS);
    designations::become_prepared(&mut t.g, emeritus);
    let copy = prepared_copy(&t, emeritus).expect("prepared");
    // Two spells first.
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    assert_eq!(prepared_copy(&t, emeritus), Some(copy));
    // The third: the prepared copy of Lightning Bolt.
    t.lands(P0, "Mountain", 1);
    t.cast(P0, copy).target(P1).go();
    t.settle();
    // Casting it unprepared Emeritus; its ability triggered on top of the Bolt.
    assert!(prepared_copy(&t, emeritus).is_none());
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    let again = prepared_copy(&t, emeritus).expect("prepared again");
    assert_ne!(again, copy);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(prepared_copy(&t, emeritus), Some(again));
}
