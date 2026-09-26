//! CR 702.72 Champion.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::{destroy, stack_triggers};
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The candidates offered by the most recent choice of objects.
fn last_choice(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn champion_exiles_another_creature_and_returns_it_when_it_leaves() {
    cr!("702.72", "702.72a");
    assert_supported("Changeling Hero");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hero = t.enter(P0, "Changeling Hero");
    t.settle();
    assert_eq!(stack_triggers(&t, "Champion").len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(hero));
    assert_eq!(t.zone(bears), Zone::Exile);
    destroy(&mut t, hero);
    t.settle();
    assert_eq!(stack_triggers(&t, "Champion").len(), 1);
    t.resolve_all();
    // It returns as a new object.
    let back = t.g.current(bears);
    assert_ne!(back, bears);
    assert_eq!(t.g.obj(back).zone, Zone::Battlefield);
    assert_eq!(t.g.obj(back).controller, P0);
}

#[test]
fn champion_sacrifices_the_permanent_unless_another_is_exiled() {
    cr!("702.72a");
    let mut t = TestGame::new(2);
    // Nothing else to champion.
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert!(!t.on_battlefield(hero));
    assert!(t.in_graveyard(P0, "Changeling Hero"));
    // Choosing not to exile the other creature also sacrifices it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[]);
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert!(!t.on_battlefield(hero));
    assert!(t.on_battlefield(bears));
}

#[test]
fn champion_an_object_exiles_only_that_kind_of_permanent_you_control() {
    cr!("702.72a");
    assert_supported("Wren's Run Packmaster");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let changeling = t.battlefield(P0, "Woodland Changeling");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their_elf = t.battlefield(P1, "Llanowar Elves");
    // "Champion an Elf."
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.enter(P0, "Wren's Run Packmaster");
    t.resolve_all();
    let offered = last_choice(&t);
    assert!(offered.contains(&Entity::Object(elves)));
    // A changeling is an Elf.
    assert!(offered.contains(&Entity::Object(changeling)));
    assert!(!offered.contains(&Entity::Object(bears)));
    assert!(!offered.contains(&Entity::Object(their_elf)));
    assert_eq!(t.zone(elves), Zone::Exile);
}

#[test]
fn the_card_returns_under_its_owners_control() {
    cr!("702.72a");
    let mut t = TestGame::new(2);
    // P0 controls P1's Grizzly Bears (Act of Treason) and champions it.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    destroy(&mut t, hero);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn a_championed_card_that_left_exile_isnt_returned() {
    cr!("702.72a", "400.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert_eq!(t.g.obj(exiled).zone, Zone::Exile);
    // Another effect puts the exiled card into its owner's graveyard.
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P1,
        mtg_engine::ability::Effect::Move {
            what: mtg_engine::ability::Sel::Target(0),
            to: mtg_engine::ability::Destination::zone(mtg_engine::ability::ZoneKind::Graveyard),
        },
        &[Entity::Object(exiled)],
    );
    destroy(&mut t, hero);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn the_two_abilities_are_linked() {
    cr!("702.72b", "607.2k");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(a)]);
    let hero1 = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    // The first hero leaving returns only the card it exiled.
    destroy(&mut t, hero1);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert_eq!(t.zone(b), Zone::Exile);
}

#[test]
fn a_champion_that_leaves_before_its_enters_ability_resolves_exiles_for_good() {
    cr!("702.72a", "702.72b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hero = t.enter(P0, "Changeling Hero");
    t.settle();
    // In response, the hero leaves: its leaves ability does nothing.
    destroy(&mut t, hero);
    t.settle();
    assert_eq!(stack_triggers(&t, "Champion").len(), 2);
    t.resolve();
    assert!(t.on_battlefield(bears));
    // Then the enters ability exiles the Bears, never to return.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn a_championed_trigger_sees_the_permanent_championed_with_this() {
    cr!("702.72c");
    assert_supported("Mistbind Clique");
    let mut t = TestGame::new(2);
    let lands = t.lands(P1, "Island", 3);
    let faerie = t.battlefield(P0, "Pestermite");
    t.answer_choose(P0, &[Entity::Object(faerie)]);
    // "When a Faerie is championed with ~, tap all lands target player controls."
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Mistbind Clique");
    t.resolve_all();
    assert_eq!(t.zone(faerie), Zone::Exile);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn a_permanent_championed_by_another_permanent_doesnt_trigger_it() {
    cr!("702.72c");
    let mut t = TestGame::new(2);
    let lands = t.lands(P1, "Island", 2);
    t.battlefield(P0, "Mistbind Clique");
    let faerie = t.battlefield(P0, "Pestermite");
    // Changeling Hero champions the Faerie: it's not championed with Mistbind Clique.
    t.answer_choose(P0, &[Entity::Object(faerie)]);
    t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert_eq!(t.zone(faerie), Zone::Exile);
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
}
