//! CR 603.2c / 603.6a / 603.2d audit: simultaneous events. One event with several
//! occurrences triggers an ability once for each, a "one or more" ability once per event
//! (or batch), permanents entering together see each other, objects that appear later in
//! the same resolution didn't exist for the earlier events, and "triggers an additional
//! time" effects that leave or enter with the objects still apply.

use crate::r600_common::compile_def;
use crate::r603_audit_common::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn wrath(t: &mut TestGame) {
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, "Wrath of God");
    t.cast(P0, w).go();
    t.resolve();
}

#[test]
fn living_death_each_step_is_its_own_event_and_returners_see_only_the_last() {
    cr!("603.2", "603.6a", "603.10a", "603.2c");
    ruling!(
        "Living Death",
        "Then all players sacrifice all creatures they control at the same time. Then all players put all creatures they exiled onto the battlefield at the same time."
    );
    supported("Living Death");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Soul Warden");
    t.graveyard(P0, "Blood Artist");
    t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    let ld = t.hand(P0, "Living Death");
    targets_n(&mut t, P0, Entity::Player(P1), 3);
    t.cast(P0, ld).go();
    t.resolve();
    // The Blood Artist on the battlefield was sacrificed with the other two creatures and
    // saw all three die. The one returning from the graveyard didn't exist when they died.
    assert_eq!(triggers_of(&t, "Blood Artist"), 3);
    // The Warden entered with the Blood Artist and the Hill Giant and saw both enter; the
    // sacrificed creatures went to the graveyard after the exile and don't come back.
    assert_eq!(triggers_of(&t, "Soul Warden"), 2);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
    assert_eq!(controlled_named(&t, P1, "Hill Giant"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20 + 3 + 2);
}

#[test]
fn a_one_or_more_dies_ability_triggers_once_per_event_and_looks_back() {
    cr!("603.2c", "603.10a");
    supported("Vengeful Townsfolk");
    let mut t = TestGame::new(2);
    let towns = t.battlefield(P0, "Vengeful Townsfolk");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let clasm = t.hand(P0, "Pyroclasm");
    t.cast(P0, clasm).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Vengeful Townsfolk"), 1);
    t.resolve_all();
    assert_eq!(t.counters(towns, counters::PLUS1), 1);
    // Dying with the others, it still sees them die (once).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vengeful Townsfolk");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    assert_eq!(triggers_of(&t, "Vengeful Townsfolk"), 1);
    // Creatures dying in separate events: once each.
    let mut t = TestGame::new(2);
    let towns = t.battlefield(P0, "Vengeful Townsfolk");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(a, None);
    t.resolve_all();
    t.g.destroy(b, None);
    t.resolve_all();
    assert_eq!(t.counters(towns, counters::PLUS1), 2);
}

#[test]
fn a_one_or_more_ability_lost_because_of_the_event_still_triggers_on_it() {
    cr!("603.10", "603.10a", "603.2c");
    // Relic Vial's wording with a "one or more" trigger: the artifact stays, but it has
    // the ability only while its controller controls a Cleric.
    let vial = compile_def(
        "Batch Vial",
        "Artifact",
        "{1}",
        "As long as you control a Cleric, this artifact has \"Whenever one or more creatures you control die, you gain 3 life.\"",
    );
    assert!(vial.is_fully_supported(), "{:?}", vial.unsupported_text());
    // While the Cleric lives, it has the ability.
    let mut t = TestGame::new(2);
    t.custom(P0, vial.clone(), Zone::Battlefield);
    t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Batch Vial"), 1);
    // A Wrath kills the Cleric: the artifact had the ability immediately before.
    let mut t = TestGame::new(2);
    let v = t.custom(P0, vial.clone(), Zone::Battlefield);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    assert!(t.on_battlefield(v));
    assert_eq!(triggers_of(&t, "Batch Vial"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // The Cleric died earlier: the artifact lost the ability before the Bears died.
    let mut t = TestGame::new(2);
    t.custom(P0, vial, Zone::Battlefield);
    let warden = t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(warden, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Batch Vial"), 1, "the Warden dying");
    t.resolve_all();
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Batch Vial"), 0);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn an_additional_trigger_effect_dying_with_the_creatures_still_applies() {
    cr!("603.2d", "603.10a");
    ruling!(
        "Teysa Karlov",
        "If a creature dying at the same time as Teysa (including Teysa itself dying) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Teysa Karlov");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    // P1's Blood Artist isn't a permanent P0 controls.
    t.battlefield(P1, "Blood Artist");
    targets_n(&mut t, P0, Entity::Player(P1), 8);
    targets_n(&mut t, P1, Entity::Player(P0), 4);
    wrath(&mut t);
    assert_eq!(
        trigger_controllers(&t, "Blood Artist"),
        [vec![P0; 8], vec![P1; 4]].concat(),
        "P0's Blood Artist triggers twice for each of its four creatures dying, P1's once"
    );
}

#[test]
fn an_ability_triggering_on_the_sacrifice_isnt_a_dies_trigger_but_a_leaves_one_is() {
    cr!("603.2d");
    ruling!(
        "Teysa Karlov",
        "An ability that triggers on an event that causes a creature to die doesn't trigger twice. For example, an ability that triggers “whenever you sacrifice a creature” triggers only once."
    );
    ruling!(
        "Teysa Karlov",
        "An ability that triggers when a creature “leaves the battlefield” will trigger twice if that creature leaves the battlefield by dying."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    t.battlefield(P0, "Mayhem Devil");
    let seer = t.battlefield(P0, "Viscera Seer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, seer, 0, &[]).expect("activate Viscera Seer");
    t.settle();
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 1);
    // Fiend Hunter's leaves ability: twice when it dies, once when it's exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    let hunter = t.battlefield(P0, "Fiend Hunter");
    t.g.destroy(hunter, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Fiend Hunter"), 2);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    let hunter = t.battlefield(P0, "Fiend Hunter");
    t.g.exile_object(hunter, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Fiend Hunter"), 1);
}

#[test]
fn an_additional_trigger_effect_entering_with_the_permanents_applies_to_them() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Panharmonicon",
        "If an artifact or creature entering the battlefield at the same time as Panharmonicon (including Panharmonicon itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Open the Vaults");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Panharmonicon");
    t.graveyard(P0, "Ichor Wellspring");
    t.lands(P0, "Plains", 6);
    let vaults = t.hand(P0, "Open the Vaults");
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, vaults).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Ichor Wellspring"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn an_enters_trigger_sees_the_counters_the_permanent_enters_with() {
    cr!("603.6d", "603.6a");
    ruling!(
        "Garruk's Uprising",
        "The same is true for replacement effects that apply to it, such as entering with one or more +1/+1 counters"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Uprising");
    t.lands(P0, "Wastes", 7);
    let big = t.hand(P0, "Endless One");
    let small = t.hand(P0, "Endless One");
    let hand = t.hand_size(P0) - 2;
    t.cast(P0, big).x(4).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Garruk's Uprising"), 1);
    t.resolve_all();
    t.cast(P0, small).x(3).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Garruk's Uprising"), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn an_enters_trigger_isnt_undone_by_the_creature_shrinking_or_leaving() {
    cr!("603.2", "113.7a");
    ruling!(
        "Garruk's Uprising",
        "Once the last ability of Garruk's Uprising has triggered, lowering the power of the creature or removing it from the battlefield won't stop you from drawing a card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Uprising");
    t.lands(P0, "Wastes", 4);
    let big = t.hand(P0, "Endless One");
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, big).x(4).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Garruk's Uprising"), 1);
    t.g.destroy(t.g.current(big), None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn an_enters_ability_with_an_intervening_if_checks_after_the_event_and_on_resolution() {
    cr!("603.4", "603.6a");
    ruling!(
        "Garruk's Uprising",
        "If you don't control a creature with power 4 or greater immediately after Garruk's Uprising enters, its first ability won't trigger. If you don't control one as the ability resolves, you don't draw a card. They don't have to be the same creature both times, however."
    );
    // No big creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Garruk's Uprising");
    t.settle();
    assert_eq!(triggers_of(&t, "Garruk's Uprising"), 0);
    // A big creature then, gone by the resolution, another one there: draws.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Craw Wurm");
    t.enter(P0, "Garruk's Uprising");
    t.settle();
    assert_eq!(triggers_of(&t, "Garruk's Uprising"), 1);
    let hand = t.hand_size(P0);
    t.g.destroy(giant, None);
    t.battlefield(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Gone and not replaced: the ability does nothing.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Craw Wurm");
    t.enter(P0, "Garruk's Uprising");
    t.settle();
    let hand = t.hand_size(P0);
    t.g.destroy(giant, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_one_or_more_ability_triggers_an_additional_time_for_teysa_dying_with_the_creatures() {
    cr!("603.2d", "603.10a", "603.2c");
    ruling!(
        "Teysa Karlov",
        "If a creature dying at the same time as Teysa (including Teysa itself dying) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Teysa Karlov");
    supported("Vengeful Townsfolk");
    // Teysa dies with the Townsfolk and a Bears: the Townsfolk's "one or more" ability
    // looks back, and so does Teysa's effect.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    t.battlefield(P0, "Vengeful Townsfolk");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Vengeful Townsfolk"), 2);
    // With Teysa still there, twice; without her, once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    let towns = t.battlefield(P0, "Vengeful Townsfolk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.counters(towns, counters::PLUS1), 2);
    let mut t = TestGame::new(2);
    let towns = t.battlefield(P0, "Vengeful Townsfolk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.counters(towns, counters::PLUS1), 1);
}

#[test]
fn an_ability_gained_because_of_the_event_doesnt_look_back_at_it() {
    cr!("603.10", "603.10a", "603.2c");
    // Threshold versions of Relic Vial: the artifacts have the abilities only once seven
    // or more cards are in their controller's graveyard.
    let vial = compile_def(
        "Threshold Vial",
        "Artifact",
        "{1}",
        "As long as seven or more cards are in your graveyard, this artifact has \"Whenever one or more creatures you control die, you gain 3 life.\"",
    );
    let urn = compile_def(
        "Threshold Urn",
        "Artifact",
        "{1}",
        "As long as seven or more cards are in your graveyard, this artifact has \"Whenever a creature you control dies, you gain 1 life.\"",
    );
    for def in [&vial, &urn] {
        assert!(def.is_fully_supported(), "{:?}", def.unsupported_text());
    }
    let mut t = TestGame::new(2);
    t.custom(P0, vial, Zone::Battlefield);
    t.custom(P0, urn, Zone::Battlefield);
    for _ in 0..5 {
        t.graveyard(P0, "Wastes");
    }
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    // The Bears dying put the seventh card into the graveyard: immediately before the
    // event the artifacts didn't have the abilities.
    assert!(t.graveyard_size(P0) >= 7);
    assert_eq!(triggers_of(&t, "Threshold Vial"), 0);
    assert_eq!(triggers_of(&t, "Threshold Urn"), 0);
    // Now they have them.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Threshold Vial"), 1);
    assert_eq!(triggers_of(&t, "Threshold Urn"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}
