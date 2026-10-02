//! "Whenever a source deals damage to ~, that source's controller [instruction]." (pattern
//! in `src/oracle/patterns/damage_source_controller.rs`): the player who controls the
//! damage's source (CR 120.1, 120.2b).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// P1 casts Shock at `target` during P0's main phase.
fn shock(t: &mut TestGame, target: ObjectId) {
    let s = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P1, s).target(Entity::Object(target)).go();
    t.resolve_all();
}

#[test]
fn source_controller_cards_compile() {
    assert_compiles(&[
        "Crag Saurian",
        "Reaper of Sheoldred",
        "Belltower Sphinx",
        "Phyrexian Obliterator",
        "Archfiend of Spite",
    ]);
}

#[test]
fn crag_saurian_the_sources_controller_gains_control_of_it() {
    cr!("120.1", "603.2");
    let mut t = TestGame::new(2);
    let saurian = t.battlefield(P0, "Crag Saurian");
    shock(&mut t, saurian);
    assert_eq!(t.obj_now(saurian).controller, P1);
    assert_eq!(t.obj_now(saurian).damage, 2);
}

#[test]
fn reaper_of_sheoldred_gives_one_poison_counter_each_time() {
    cr!("120.1", "603.2");
    ruling!(
        "Reaper of Sheoldred",
        "not one poison counter for each 1 damage dealt."
    );
    let mut t = TestGame::new(2);
    let reaper = t.battlefield(P0, "Reaper of Sheoldred");
    shock(&mut t, reaper);
    assert_eq!(t.g.player(P1).poison(), 1);
    assert_eq!(t.g.player(P0).poison(), 0);
}

#[test]
fn belltower_sphinx_the_sources_controller_mills_that_many_cards() {
    cr!("120.1", "701.17a");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Belltower Sphinx");
    for _ in 0..5 {
        t.library_top(P1, "Island");
    }
    let before = t.library_size(P1);
    shock(&mut t, sphinx);
    assert_eq!(t.library_size(P1), before - 2);
}

#[test]
fn phyrexian_obliterator_blockers_dealt_lethal_damage_die_first() {
    cr!("704.3", "701.21a");
    ruling!(
        "Phyrexian Obliterator",
        "those creatures will be destroyed before that player chooses permanents to sacrifice."
    );
    let mut t = TestGame::new(2);
    let obliterator = t.battlefield(P0, "Phyrexian Obliterator");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(obliterator, Entity::Player(P1))], &[(bears, obliterator)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The Bears dealt 2 damage: two of the Forests were sacrificed.
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 1);
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn archfiend_of_spite_without_enough_permanents_they_lose_life() {
    cr!("118.12a", "120.1");
    ruling!(
        "Archfiend of Spite",
        "If the player doesn't control enough permanents that they can sacrifice, they must lose life."
    );
    let mut t = TestGame::new(2);
    let archfiend = t.battlefield(P0, "Archfiend of Spite");
    let life = t.life(P1);
    // P1 controls only the Mountain that paid for Shock.
    shock(&mut t, archfiend);
    assert_eq!(t.life(P1), life - 2);
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 1);
}

#[test]
fn archfiend_of_spite_ignores_damage_from_your_own_sources() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let archfiend = t.battlefield(P0, "Archfiend of Spite");
    let s = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    t.cast(P0, s).target(Entity::Object(archfiend)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), life);
    assert_eq!(t.obj_now(archfiend).damage, 2);
}
