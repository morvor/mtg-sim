//! Rulings batch S04 — crew (CR 702.122): "Crew N" means "Tap any number of other
//! untapped creatures you control with total power N or greater: This permanent becomes
//! an artifact creature until end of turn."
//!
//! Most of these rulings are printed twice (with straight and with curly apostrophes, on
//! different Vehicles); the straight versions are cited with Smuggler's Copter by the
//! keyword tests, the curly ones here with Consulate Dreadnought, Dusk Legion Dreadnought,
//! Sleek Schooner, and Irontread Crusher.

use crate::r_s01_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s04_common::*;
use mtg_engine::ability::{Duration, Effect, Sel};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the object has any creature type.
fn has_creature_type(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .subtypes
        .iter()
        .any(|s| is_creature_type(s))
}

#[test]
fn a_vehicle_must_be_crewed_by_the_beginning_of_combat_to_attack() {
    cr!("702.122a", "508.1", "508.1a", "117.3b", "117.3c");
    ruling!(
        "Cultivator's Caravan",
        "For a Vehicle to be able to attack, it must be a creature as the declare attackers step begins, so the latest you can activate its crew ability to attack with it is during the beginning of combat step."
    );
    supported("Cultivator's Caravan");
    // Cultivator's Caravan: a 5/5 Vehicle with crew 3.
    // Crewed during the beginning of combat step, it can attack.
    let mut t = TestGame::new(2);
    let caravan = t.battlefield(P0, "Cultivator's Caravan");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, caravan, &[giant]));
    t.resolve();
    attack_with(&mut t, &[(caravan, Entity::Player(P1))]);
    assert!(t.g.is_attacking(caravan));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);

    // Crewed during the declare attackers step, it's too late: attackers were declared.
    let mut t = TestGame::new(2);
    let caravan = t.battlefield(P0, "Cultivator's Caravan");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(crew(&mut t, P0, caravan, &[giant]));
    t.resolve();
    assert!(is_creature(&t, caravan));
    assert!(!t.g.is_attacking(caravan));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_vehicle_must_be_crewed_by_the_declare_attackers_step_to_block() {
    cr!("702.122a", "509.1", "509.1a");
    ruling!(
        "Cultivator's Caravan",
        "For a Vehicle to be able to block, it must be a creature as the declare blockers step begins, so the latest you can activate its crew ability to block with it is during the declare attackers step."
    );
    // Crewed by the defending player during the declare attackers step, it can block.
    let mut t = TestGame::new(2);
    let caravan = t.battlefield(P1, "Cultivator's Caravan");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(crew(&mut t, P1, caravan, &[giant]));
    t.resolve();
    block_and_finish(&mut t, P1, &[(caravan, bears)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));

    // Crewed during the declare blockers step, it's too late: blockers were declared.
    let mut t = TestGame::new(2);
    let caravan = t.battlefield(P1, "Cultivator's Caravan");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(crew(&mut t, P1, caravan, &[giant]));
    t.resolve();
    assert!(is_creature(&t, caravan));
    assert!(!t.g.is_blocking(caravan));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn players_may_act_after_the_crew_ability_resolves_before_attackers_are_declared() {
    cr!("702.122a", "117.3b", "117.3d", "508.1");
    ruling!(
        "Cultivator's Caravan",
        "In either case, players may take actions after the crew ability resolves but before the Vehicle has been declared as an attacking or blocking creature."
    );
    let mut t = TestGame::new(2);
    let caravan = t.battlefield(P0, "Cultivator's Caravan");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, caravan, &[giant]));
    t.resolve();
    assert!(is_creature(&t, caravan));
    // The crewed Vehicle is a creature: when P0 passes priority, P1 destroys it with
    // Murder, still in the beginning of combat step.
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: murder,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[Entity::Object(caravan)]);
    attack_with(&mut t, &[(caravan, Entity::Player(P1))]);
    assert!(t.in_graveyard(P1, "Murder"));
    assert!(t.in_graveyard(P0, "Cultivator's Caravan"));
    assert!(t.g.attackers().is_empty());
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

/// Activates the crew ability of `vehicle` tapping `crewers`, asserting that no player
/// received priority (or was asked anything but the crew payment) until it was paid.
fn crew_without_interruption(t: &mut TestGame, vehicle: ObjectId, crewers: &[ObjectId]) {
    let from = t.asked().len();
    assert!(crew(t, P0, vehicle, crewers));
    let asked = asked_since(t, from);
    assert!(
        asked
            .iter()
            .all(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })),
        "decisions during the activation: {asked:?}"
    );
    // The cost is paid and the ability is on the stack.
    assert!(crewers.iter().all(|c| t.obj(*c).tapped));
    assert_eq!(on_stack(t, "Crew"), 1);
}

#[test]
fn no_one_can_act_while_a_crew_ability_is_activated_and_paid_for() {
    cr!("702.122a", "602.2", "602.2b", "601.2h", "601.2i");
    ruling!(
        "Smuggler's Copter",
        "Once a player announces that they are activating a crew ability, no player may take other actions until the ability has been paid for. Notably, players can't try to stop the ability by changing a creature's power or by removing or tapping a creature."
    );
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crew_without_interruption(&mut t, copter, &[bears]);
    // Removing the creature that crewed it doesn't stop the ability.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve();
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (3, 3));
}

#[test]
fn changing_a_crew_members_power_in_response_doesnt_stop_the_ability() {
    cr!("702.122a", "602.2b", "601.2h");
    ruling!(
        "Sleek Schooner",
        "Once a player announces that they are activating a crew ability, no player may take other actions until the ability has been paid for. Notably, players can’t try to stop the ability by changing a creature’s power or by removing or tapping a creature."
    );
    supported("Sleek Schooner");
    // Sleek Schooner: a 4/3 Vehicle with crew 1.
    let mut t = TestGame::new(2);
    let schooner = t.battlefield(P0, "Sleek Schooner");
    let elves = t.battlefield(P0, "Llanowar Elves");
    crew_without_interruption(&mut t, schooner, &[elves]);
    // In response, the creature that crewed it becomes 0/2 (Sudden Spoiling): the cost
    // was already paid, and the ability resolves.
    t.lands(P1, "Swamp", 3);
    let spoil = t.hand(P1, "Sudden Spoiling");
    t.cast(P1, spoil).target(P0).go();
    t.resolve();
    assert_eq!(t.pt(elves), (0, 2));
    t.resolve();
    assert!(is_creature(&t, schooner));
    assert_eq!(t.pt(schooner), (4, 3));
}

#[test]
fn a_crewed_vehicle_has_no_creature_type() {
    cr!("702.122a", "205.3g", "205.3m", "301.7");
    ruling!(
        "Sleek Schooner",
        "Vehicle is an artifact type, not a creature type. A Vehicle that’s crewed won’t normally have any creature type."
    );
    assert!(!is_creature_type("Vehicle"));
    let mut t = TestGame::new(2);
    let schooner = t.battlefield(P0, "Sleek Schooner");
    // Two Elves: Coat of Arms gives each of them +1/+1, but not the crewed Vehicle, which
    // shares no creature type with them.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let other_elves = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Coat of Arms");
    assert!(crew(&mut t, P0, schooner, &[elves]));
    t.resolve();
    assert!(is_creature(&t, schooner));
    assert!(t.obj_now(schooner).chars.has_subtype("Vehicle"));
    assert!(!has_creature_type(&t, schooner));
    assert_eq!(t.pt(schooner), (4, 3));
    assert_eq!(t.pt(other_elves), (2, 2));
}

#[test]
fn a_crewed_vehicle_behaves_like_any_other_artifact_creature() {
    cr!("702.122a", "302.6", "508.1a", "509.1a");
    ruling!(
        "Dusk Legion Dreadnought",
        "Once a Vehicle becomes a creature, it behaves exactly like any other artifact creature. It can’t attack unless you’ve controlled it continuously since your turn began, it can block if it’s untapped, it can be tapped to pay a Vehicle’s crew cost, and so on."
    );
    supported("Dusk Legion Dreadnought");
    supported("Irontread Crusher");
    // Dusk Legion Dreadnought: a 4/6 Vehicle with vigilance and crew 2.
    let mut t = TestGame::new(2);
    let dread = t.battlefield_sick(P0, "Dusk Legion Dreadnought");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, dread, &[giant]));
    t.resolve();
    assert!(t.obj_now(dread).is(CardType::Artifact) && is_creature(&t, dread));
    // It came under P0's control this turn: it can't attack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, dread));
    // A crewed Vehicle can crew another Vehicle (Irontread Crusher: crew 3).
    let crusher = t.battlefield(P0, "Irontread Crusher");
    assert!(crew(&mut t, P0, crusher, &[dread]));
    assert!(t.obj(dread).tapped);
    t.resolve();
    assert!(is_creature(&t, crusher));

    // Crewed on the opponent's turn, it can block while it's untapped.
    let mut t = TestGame::new(2);
    let dread = t.battlefield(P1, "Dusk Legion Dreadnought");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(crew(&mut t, P1, dread, &[giant]));
    t.resolve();
    assert!(t.g.can_block_at_all(dread));
    t.g.tap(dread);
    assert!(!t.g.can_block_at_all(dread));
}

#[test]
fn crewing_an_artifact_creature_vehicle_again_changes_nothing() {
    cr!("702.122a", "613.4b");
    ruling!(
        "Sleek Schooner",
        "You may activate a crew ability of a Vehicle even if it’s already an artifact creature. Doing so has no effect on the Vehicle. It doesn’t change its power and toughness."
    );
    supported("Ensoul Artifact");
    let mut t = TestGame::new(2);
    let schooner = t.battlefield(P0, "Sleek Schooner");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Ensoul Artifact makes it a 5/5 artifact creature.
    t.lands(P0, "Island", 2);
    let ensoul = t.hand(P0, "Ensoul Artifact");
    t.cast(P0, ensoul).target(schooner).go();
    t.resolve_all();
    assert!(is_creature(&t, schooner));
    assert_eq!(t.pt(schooner), (5, 5));
    // Crewing it anyway (twice) is legal and changes nothing.
    assert!(crew(&mut t, P0, schooner, &[elves]));
    t.resolve();
    assert!(crew(&mut t, P0, schooner, &[bears]));
    t.resolve();
    assert!(t.obj(elves).tapped && t.obj(bears).tapped);
    assert_eq!(t.pt(schooner), (5, 5));
    assert!(t.obj_now(schooner).is(CardType::Artifact) && is_creature(&t, schooner));
}

#[test]
fn a_vehicle_becoming_a_creature_doesnt_enter_the_battlefield() {
    cr!("702.122a", "603.6a", "110.1");
    ruling!(
        "Irontread Crusher",
        "When a Vehicle becomes a creature, that doesn’t count as having a creature enter the battlefield. The permanent was already on the battlefield; it only changed its types. Abilities that trigger whenever a creature enters the battlefield won’t trigger."
    );
    let mut t = TestGame::new(2);
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let crusher = t.battlefield(P0, "Irontread Crusher");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, crusher, &[giant]));
    t.resolve_all();
    assert!(is_creature(&t, crusher));
    assert_eq!(t.life(P0), 20);
    assert!(t.g.stack.is_empty());
    // A creature that does enter triggers it.
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn an_effect_that_sets_a_vehicles_power_and_toughness_overwrites_the_printed_ones() {
    cr!("702.122a", "208.3", "613.4b");
    ruling!(
        "Smuggler's Copter",
        "If an effect causes a Vehicle to become an artifact creature with a specified power and toughness, that effect overwrites the Vehicle's printed power and toughness."
    );
    ruling!(
        "Consulate Dreadnought",
        "If an effect causes a Vehicle to become an artifact creature with a specified power and toughness, that effect overwrites the Vehicle’s printed power and toughness."
    );
    supported("Consulate Dreadnought");
    // Ensoul Artifact: "Enchanted artifact is a creature with base power and toughness 5/5
    // in addition to its other types."
    for (name, printed) in [("Smuggler's Copter", (3, 3)), ("Consulate Dreadnought", (7, 11))] {
        // Crewed, it has its printed power and toughness.
        let mut t = TestGame::new(2);
        let vehicle = t.battlefield(P0, name);
        let wurm = t.battlefield(P0, "Craw Wurm");
        assert!(crew(&mut t, P0, vehicle, &[wurm]));
        t.resolve();
        assert_eq!(t.pt(vehicle), printed, "{name}");
        // Enchanted with Ensoul Artifact, it's a 5/5 artifact creature instead.
        let mut t = TestGame::new(2);
        let vehicle = t.battlefield(P0, name);
        t.lands(P0, "Island", 2);
        let ensoul = t.hand(P0, "Ensoul Artifact");
        t.cast(P0, ensoul).target(vehicle).go();
        t.resolve_all();
        assert!(is_creature(&t, vehicle));
        assert!(t.obj_now(vehicle).is(CardType::Artifact));
        assert_eq!(t.pt(vehicle), (5, 5), "{name}");
        // Crewing it doesn't bring back the printed power and toughness.
        let wurm = t.battlefield(P0, "Craw Wurm");
        let giant = t.battlefield(P0, "Hill Giant");
        assert!(crew(&mut t, P0, vehicle, &[wurm, giant]));
        t.resolve();
        assert_eq!(t.pt(vehicle), (5, 5), "{name}");
    }
}

#[test]
fn a_vehicle_isnt_a_creature_until_crewed_and_then_has_its_printed_power_and_toughness() {
    cr!("702.122a", "301.7", "208.3");
    ruling!(
        "Consulate Dreadnought",
        "Each Vehicle is printed with a power and toughness, but it’s not a creature. If it becomes a creature (most likely through its crew ability), it will have that power and toughness."
    );
    let mut t = TestGame::new(2);
    // Consulate Dreadnought: a 7/11 Vehicle with crew 6.
    let dread = t.battlefield(P0, "Consulate Dreadnought");
    assert!(!is_creature(&t, dread));
    // As a noncreature permanent it has no power or toughness (CR 208.3), and "destroy
    // target creature" can't target it.
    assert_eq!(t.obj_now(dread).chars.power, None);
    assert_eq!(t.obj_now(dread).chars.toughness, None);
    assert!(!spell_targets(&mut t, P1, "Murder").contains(&Entity::Object(dread)));
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    // Craw Wurm alone (power 6) is enough.
    assert!(crew(&mut t, P0, dread, &[wurm]));
    t.resolve();
    assert!(!t.obj(giant).tapped);
    assert!(is_creature(&t, dread));
    assert_eq!(t.pt(dread), (7, 11));
    assert!(spell_targets(&mut t, P1, "Murder").contains(&Entity::Object(dread)));
}

#[test]
fn creatures_that_crewed_a_vehicle_arent_affected_by_what_affects_it() {
    cr!("702.122a", "702.122c");
    ruling!(
        "Irontread Crusher",
        "Creatures that crew a Vehicle aren’t attached to it or related in any other way. Effects that affect the Vehicle, such as by destroying it or giving it a +1/+1 counter, don’t affect the creatures that crewed it."
    );
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Irontread Crusher");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, crusher, &[giant]));
    t.resolve();
    assert_eq!(t.obj(giant).attached_to, None);
    assert_eq!(t.obj_now(crusher).attached_to, None);
    t.g.add_counters(Entity::Object(crusher), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(crusher), (7, 7));
    assert_eq!(t.counters(giant, "+1/+1"), 0);
    assert_eq!(t.pt(giant), (3, 3));
    // Destroying the Vehicle doesn't destroy the creature that crewed it.
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(crusher).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Irontread Crusher"));
    assert!(t.on_battlefield(giant));
}

#[test]
fn a_copy_of_a_crewed_vehicle_isnt_a_creature() {
    cr!("702.122a", "707.2", "613.2a");
    ruling!(
        "Dusk Legion Dreadnought",
        "If a permanent becomes a copy of a Vehicle, the copy won’t be a creature, even if the Vehicle it’s copying has become an artifact creature."
    );
    supported("Sculpting Steel");
    let mut t = TestGame::new(2);
    let dread = t.battlefield(P0, "Dusk Legion Dreadnought");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, dread, &[giant]));
    t.resolve();
    assert!(is_creature(&t, dread));
    // Sculpting Steel enters as a copy of the crewed Vehicle: a noncreature Vehicle.
    t.lands(P0, "Wastes", 3);
    let steel = t.hand(P0, "Sculpting Steel");
    t.cast(P0, steel).go();
    t.answer_choose(P0, &[Entity::Object(dread)]);
    t.resolve_all();
    let copy = t.g.current(steel);
    assert_eq!(t.obj(copy).chars.name, "Dusk Legion Dreadnought");
    assert!(t.obj(copy).is(CardType::Artifact));
    assert!(t.obj(copy).chars.has_subtype("Vehicle"));
    assert!(!is_creature(&t, copy));
    // An artifact that becomes a copy of the crewed Vehicle isn't a creature either.
    let stone = t.battlefield(P0, "Mind Stone");
    run_with(
        &mut t,
        P0,
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(stone), Entity::Object(dread)],
    );
    assert_eq!(t.obj_now(stone).chars.name, "Dusk Legion Dreadnought");
    assert!(!is_creature(&t, stone));
    assert!(is_creature(&t, dread));
}
