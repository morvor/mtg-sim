//! CR 614.12 audit: replacement effects that modify how a permanent enters are found and
//! applied from the permanent as it would exist on the battlefield: under the player who
//! will control it (whoever puts it there), with its own static abilities and the
//! continuous effects that already exist, and not with the effects of permanents entering
//! at the same time.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_grave_troll_returned_by_an_opponents_oblivion_ring_counts_its_owners_graveyard() {
    cr!("614.12", "614.1c", "109.5");
    supported("Golgari Grave-Troll");
    supported("Oblivion Ring");
    let mut t = TestGame::new(2);
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    graveyard_n(&mut t, P1, "Grizzly Bears", 1);
    let troll = t.enter(P0, "Golgari Grave-Troll");
    assert_eq!(plus_counters(&t, troll), 3);
    // P1's Oblivion Ring exiles the Troll.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_targets(P1, &[obj(troll)]);
    cast_resolve(&mut t, P1, "Oblivion Ring", &[]);
    assert!(t.in_exile("Golgari Grave-Troll"));
    // P0 destroys the Ring: P1's ability returns the Troll under its owner's control. "Your
    // graveyard" is that of the player it enters under: P0's three creature cards.
    let ring = t.named_on_battlefield("Oblivion Ring")[0];
    cast_resolve(&mut t, P0, "Disenchant", &[obj(ring)]);
    let troll = t.g.current(troll);
    assert!(t.on_battlefield(troll));
    assert_eq!(controller(&mut t, troll), P0);
    assert_eq!(plus_counters(&t, troll), 3);
}

#[test]
fn a_clone_copying_a_grave_troll_counts_its_own_controllers_graveyard() {
    cr!("614.12", "614.1c", "707.2");
    let mut t = TestGame::new(2);
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    graveyard_n(&mut t, P1, "Grizzly Bears", 2);
    let troll = t.enter(P0, "Golgari Grave-Troll");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(troll)]);
    let clone = cast_resolve(&mut t, P1, "Clone", &[]);
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Golgari Grave-Troll");
    assert_eq!(plus_counters(&t, clone), 2);
}

#[test]
fn ashaya_is_a_nonbasic_land_as_it_enters_so_archon_of_emeria_taps_it() {
    cr!("614.12", "614.1d");
    supported("Archon of Emeria");
    supported("Ashaya, Soul of the Wild");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P0, "Archon of Emeria");
    // Before Ashaya, a creature enters untapped.
    let early = t.enter(P1, "Grizzly Bears");
    assert!(!is_tapped(&t, early));
    // Ashaya's own static ability makes it a Forest land (not basic) on the battlefield.
    let ashaya = cast_resolve(&mut t, P1, "Ashaya, Soul of the Wild", &[]);
    let ashaya = t.g.current(ashaya);
    assert!(t.obj_now(ashaya).is(CardType::Land));
    assert!(is_tapped(&t, ashaya));
    // Now a nontoken creature entering under P1's control is a Forest land too, by an
    // effect that already exists: it enters tapped.
    let bears = t.enter(P1, "Grizzly Bears");
    assert!(t.obj_now(bears).is(CardType::Land));
    assert!(is_tapped(&t, bears));
}

#[test]
fn an_enchantment_entering_under_opalescence_is_a_creature_thalia_taps() {
    cr!("614.12", "614.1d");
    supported("Opalescence");
    supported("Thalia, Heretic Cathar");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P1, "Opalescence");
    t.battlefield(P0, "Thalia, Heretic Cathar");
    let anthem = cast_resolve(&mut t, P1, "Glorious Anthem", &[]);
    let anthem = t.g.current(anthem);
    assert!(t.obj_now(anthem).is(CardType::Creature));
    assert!(is_tapped(&t, anthem));
}

#[test]
fn thalia_returning_with_an_opponents_creatures_doesnt_tap_them() {
    cr!("614.12");
    supported("Living Death");
    supported("Thalia, Heretic Cathar");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.graveyard(P0, "Thalia, Heretic Cathar");
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_resolve(&mut t, P1, "Living Death", &[]);
    let thalia = t.named_on_battlefield("Thalia, Heretic Cathar");
    assert_eq!(thalia.len(), 1);
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    // Thalia entered at the same time: its effect didn't exist yet.
    assert!(!is_tapped(&t, bears));
    // A creature P1 casts afterwards enters tapped.
    let later = cast_resolve(&mut t, P1, "Grizzly Bears", &[]);
    assert!(is_tapped(&t, t.g.current(later)));
}

#[test]
fn corpsejack_returning_with_spike_feeder_doesnt_double_its_counters() {
    cr!("614.12", "614.1c", "614.16");
    supported("Living Death");
    supported("Corpsejack Menace");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Corpsejack Menace");
    let feeder = t.graveyard(P0, "Spike Feeder");
    cast_resolve(&mut t, P0, "Living Death", &[]);
    let feeder = t.g.current(feeder);
    assert!(t.on_battlefield(feeder));
    assert_eq!(plus_counters(&t, feeder), 2);
    // A Spike Feeder entering after Corpsejack Menace gets four.
    let later = cast_resolve(&mut t, P0, "Spike Feeder", &[]);
    assert_eq!(plus_counters(&t, later), 4);
}

#[test]
fn spike_feeder_entering_under_humility_has_no_enters_with_counters_ability() {
    cr!("614.12", "614.1c", "613.1f");
    supported("Humility");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    let feeder = cast_resolve(&mut t, P0, "Spike Feeder", &[]);
    let feeder = t.g.current(feeder);
    assert!(t.on_battlefield(feeder));
    assert_eq!(plus_counters(&t, feeder), 0);
    assert_eq!(t.pt(feeder), (1, 1));
}
