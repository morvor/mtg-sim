//! CR 702.15 Lifelink.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn lifelink_damage_gains_its_controller_that_much_life() {
    cr!("702.15", "702.15a", "702.15b", "120.3f");
    assert_supported("Child of Night");
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Night");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(child, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    // Damage to a creature counts too, in addition to its other results.
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Night");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[(child, bears)]);
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(bears));
    // A static ability: granting it works the same way.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Whip of Erebos");
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn prevented_damage_isnt_dealt_so_lifelink_gains_nothing() {
    cr!("702.15b", "702.16e");
    let mut t = TestGame::new(2);
    // Absolute Virtue: "You have protection from each of your opponents."
    t.battlefield(P0, "Absolute Virtue");
    let child = t.battlefield(P1, "Child of Night");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(child, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    // Unprevented, it would have: the same attack against an unprotected player.
    let mut t = TestGame::new(2);
    let child = t.battlefield(P1, "Child of Night");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(child, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn lifelink_gains_life_for_the_sources_controller_not_its_owner() {
    cr!("702.15b");
    assert_supported("Act of Treason");
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Night");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(child).go();
    t.resolve_all();
    assert_eq!(t.obj_now(child).controller, P1);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(child, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn lifelink_source_without_a_controller_gains_its_owner_life() {
    cr!("702.15b", "702.15d");
    let mut t = TestGame::new(2);
    // A card in a graveyard has no controller; if it deals damage, its owner gains life.
    let child = t.graveyard(P1, "Child of Night");
    assert!(t.obj_now(child).has_keyword(KeywordKind::Lifelink));
    t.g.deal_damage(child, Entity::Player(P0), 2, false);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn lifelink_uses_last_known_information_of_a_source_that_left() {
    cr!("702.15c");
    assert_supported("Mogg Fanatic");
    let mut t = TestGame::new(2);
    let fanatic = t.battlefield(P0, "Mogg Fanatic");
    // "Creatures you control have lifelink."
    t.battlefield(P0, "Whip of Erebos");
    // "Sacrifice this creature: It deals 1 damage to any target." As the ability resolves,
    // the Fanatic is in the graveyard; it had lifelink when it was last on the battlefield.
    t.activate(P0, fanatic, 0, &[Entity::Player(P1)]).unwrap();
    assert!(t.in_graveyard(P0, "Mogg Fanatic"));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn lifelink_works_from_any_zone() {
    cr!("702.15d");
    assert_supported("Heartflame Duelist");
    let mut t = TestGame::new(2);
    // "Instant and sorcery spells you control have lifelink."
    t.battlefield(P0, "Heartflame Duelist");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    // An opponent's spells don't have it.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn each_lifelink_source_is_a_separate_life_gain_event() {
    cr!("702.15e", "119.9");
    ruling!(
        "Ajani's Pridemate",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(pridemate, "+1/+1"), 2);
}

#[test]
fn one_lifelink_source_damaging_several_recipients_is_one_event() {
    cr!("702.15e");
    ruling!(
        "Ajani's Pridemate",
        "if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time"
    );
    assert_supported("Shadowspear");
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Shadowspear: +1/+1, trample and lifelink.
    let spear = t.battlefield(P0, "Shadowspear");
    t.g.attach(spear, Entity::Object(bears));
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(goblin, bears)]);
    t.resolve_all();
    // 3 damage split between the 1/1 blocker and the player: one life gain event of 3.
    assert!(!t.on_battlefield(goblin));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}

#[test]
fn multiple_instances_of_lifelink_are_redundant() {
    cr!("702.15f");
    ruling!(
        "Behemoth Sledge",
        "Multiple instances of lifelink on the same creature are redundant."
    );
    assert_supported("Behemoth Sledge");
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let child = t.battlefield(P0, "Child of Night");
    // "Equipped creature gets +2/+2 and has trample and lifelink."
    let sledge = t.battlefield(P0, "Behemoth Sledge");
    t.g.attach(sledge, Entity::Object(child));
    t.g.recompute();
    assert_eq!(keyword_count(&t, child, KeywordKind::Lifelink), 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(child, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 4 damage: gain 4, once.
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}
