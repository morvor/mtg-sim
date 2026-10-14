//! CR 702.184 Station (`kw/station.rs`; station cards are CR 721, `tests/cr/r721_*`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Wurmwall Sweeper ({2} Artifact — Spacecraft): "Station. 4+ | Flying" with a 2/2 box.
const SWEEPER: &str = "Wurmwall Sweeper";

/// Activates the station ability of `station`, tapping `creature`.
fn station(t: &mut TestGame, station: ObjectId, creature: ObjectId) -> bool {
    t.answer_choose(P0, &[Entity::Object(creature)]);
    let ok = t.activate(P0, station, 0, &[]).is_ok();
    t.clear_answers();
    ok
}

fn charge(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::CHARGE)
}

fn modify_pt(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    run(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn station_taps_another_creature_for_charge_counters_equal_to_its_power() {
    cr!("702.184a");
    ruling!(
        "Atmospheric Greenhouse",
        "The station keyword means “Tap another untapped creature you control: Put a number of charge counters on this permanent equal to the tapped creature’s power. Activate only as a sorcery.”"
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    let giant = t.battlefield(P0, "Hill Giant");
    // The station ability is an activated ability.
    assert!(t
        .obj(ship)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_))));
    assert!(station(&mut t, ship, giant));
    assert!(t.obj(giant).tapped);
    t.resolve_all();
    // Hill Giant's power is 3.
    assert_eq!(charge(&t, ship), 3);
    // A tapped creature can't be tapped again, and an opponent's creature can't be tapped.
    assert!(!station(&mut t, ship, giant));
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(!station(&mut t, ship, theirs));
    // With 4 counters it's a 2/2 flying artifact creature; it can't tap itself.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(station(&mut t, ship, bears));
    t.resolve_all();
    assert_eq!(charge(&t, ship), 5);
    assert!(t.obj(ship).is_creature());
    assert!(!station(&mut t, ship, ship));
    assert_eq!(charge(&t, ship), 5);
}

#[test]
fn station_only_as_a_sorcery() {
    cr!("702.184a");
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    let giant = t.battlefield(P0, "Hill Giant");
    // Not during combat, not on an opponent's turn, not while the stack isn't empty.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!station(&mut t, ship, giant));
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(!station(&mut t, ship, giant));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(giant).go();
    assert!(!station(&mut t, ship, giant));
    t.resolve_all();
    assert!(station(&mut t, ship, giant));
    t.resolve_all();
    // Giant Growth made it 6/6 this turn.
    assert_eq!(charge(&t, ship), 6);
}

#[test]
fn a_creature_that_left_counts_with_its_last_known_power() {
    cr!("702.184a");
    ruling!(
        "Atmospheric Greenhouse",
        "If that creature isn’t on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(station(&mut t, ship, giant));
    // In response, the Giant gets +2/+0 and then leaves the battlefield.
    modify_pt(&mut t, giant, 2, 0);
    run(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(giant)],
    );
    assert!(!t.on_battlefield(giant));
    t.resolve_all();
    assert_eq!(charge(&t, ship), 5);
}

#[test]
fn a_creature_with_negative_power_puts_no_counters() {
    cr!("702.184a");
    ruling!(
        "Atmospheric Greenhouse",
        "If the tapped creature has negative power, no charge counters are put onto or removed from the permanent with station."
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    t.g.objects[ship.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify_pt(&mut t, bears, -4, 0);
    assert!(station(&mut t, ship, bears));
    t.resolve_all();
    assert_eq!(charge(&t, ship), 2);
}

#[test]
fn a_station_card_has_station_symbols_that_are_keyword_abilities() {
    cr!("702.184b");
    ruling!(
        "Atmospheric Greenhouse",
        "A station card is a card with the station keyword ability."
    );
    // Lumen-Class Frigate: "Station. 2+ | Other creatures you control get +1/+1. 12+ |
    // Flying, lifelink" (3/5 box at 12+).
    let frigate = card("Lumen-Class Frigate");
    assert!(frigate.front().chars.has_keyword(KeywordKind::Station));
    assert_supported(&["Lumen-Class Frigate", SWEEPER, "Wedgelight Rammer"]);
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Lumen-Class Frigate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    assert!(station(&mut t, ship, bears));
    t.resolve_all();
    // Its 2+ symbol's ability: other creatures get +1/+1.
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.obj(ship).is_creature());
    t.g.objects[ship.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), 12);
    t.recompute();
    assert!(t.obj(ship).is_creature());
    assert_eq!(t.pt(ship), (3, 5));
    assert!(t.obj(ship).has_keyword(KeywordKind::Flying));
}

#[test]
fn static_abilities_may_make_it_use_toughness() {
    cr!("702.184c");
    ruling!(
        "Tapestry Warden",
        "They change only the amount of combat damage the creature assigns and how many counters are put on a permanent with station"
    );
    assert_supported(&["Tapestry Warden"]);
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    t.battlefield(P0, "Tapestry Warden");
    // Horned Turtle is a 1/4: it stations using its toughness.
    let turtle = t.battlefield(P0, "Horned Turtle");
    assert!(station(&mut t, ship, turtle));
    t.resolve_all();
    assert_eq!(charge(&t, ship), 4);
    // Its power is unchanged.
    assert_eq!(t.pt(turtle), (1, 4));
    // Hill Giant (3/3) doesn't have toughness greater than its power.
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(station(&mut t, ship, giant));
    t.resolve_all();
    assert_eq!(charge(&t, ship), 7);
}

#[test]
fn the_static_ability_applies_as_the_station_ability_resolves() {
    cr!("702.184c");
    ruling!(
        "Tapestry Warden",
        "but you no longer control Tapestry Warden at the time that ability resolves, use the power of the creature tapped"
    );
    ruling!(
        "Tapestry Warden",
        "check the characteristics of that creature as it last existed on the battlefield. If its toughness was greater than its power, use its toughness"
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    let warden = t.battlefield(P0, "Tapestry Warden");
    let turtle = t.battlefield(P0, "Horned Turtle");
    assert!(station(&mut t, ship, turtle));
    // The Warden leaves before the ability resolves: the Turtle's power counts.
    run(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(warden)],
    );
    t.resolve_all();
    assert_eq!(charge(&t, ship), 1);
    // With the Warden around, a tapped creature that left counts as it last existed.
    t.battlefield(P0, "Tapestry Warden");
    let turtle2 = t.battlefield(P0, "Horned Turtle");
    assert!(station(&mut t, ship, turtle2));
    run(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(turtle2)],
    );
    t.resolve_all();
    assert_eq!(charge(&t, ship), 5);
}

#[test]
fn static_abilities_may_make_it_count_more_power() {
    cr!("702.184c");
    // Stoic Star-Captain: "Each creature you control crews Vehicles and stations
    // permanents as though its power were 2 greater."
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, SWEEPER);
    let captain = t.battlefield(P0, "Stoic Star-Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(station(&mut t, ship, bears));
    t.resolve_all();
    assert_eq!(charge(&t, ship), 4);
    assert_eq!(t.pt(bears), (2, 2));
    // It applies to the Captain itself too (power 2, stations for 4).
    let ship2 = t.battlefield(P0, SWEEPER);
    assert!(station(&mut t, ship2, captain));
    t.resolve_all();
    assert_eq!(charge(&t, ship2), 4);
    // And it crews Vehicles that way: a 2/2 alone crews Cultivator's Caravan (Crew 3).
    let caravan = t.battlefield(P0, "Cultivator's Caravan");
    let bears2 = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears2)]);
    t.activate(P0, caravan, 1, &[]).unwrap();
    assert!(t.obj(bears2).tapped);
    t.resolve_all();
    assert!(t.obj(caravan).is_creature());
}
