//! Rulings batch P214 — rulings of cards with an ability the compiler doesn't support
//! yet, about their other (supported) abilities: Rohirrim Chargers' exert, Domri's static
//! ability and fight ability, The Tarrasque's fight trigger, Strax's Glory of Battle.

use crate::r_p214_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::*;

use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Asserts that `name` has a compiled ability whose text contains `needle`.
fn has_ability(name: &str, needle: &str) {
    let c = card(name);
    assert!(
        c.front()
            .chars
            .abilities
            .iter()
            .any(|a| a.text.contains(needle)),
        "{name}: no compiled ability containing {needle:?}"
    );
}

/// Asserts that `name`'s only unsupported text is the one block containing `needle` (an
/// ability the rulings tested here don't concern).
fn only_unsupported(name: &str, needle: &str) {
    let c = card(name);
    let u = c.unsupported_text();
    assert_eq!(u.len(), 1, "{name}: {u:?}");
    assert!(u[0].contains(needle), "{name}: {u:?}");
}

/// Power of the Rohirrim Chargers on the battlefield.
fn chargers_power(g: &Game) -> i32 {
    let id = g.find_in_zone(Zone::Battlefield, "Rohirrim Chargers")[0];
    g.obj(id).power()
}

#[test]
fn rohirrim_chargers_is_exerted_as_it_is_declared_as_an_attacker() {
    cr!("508.1g", "701.43d", "509.1");
    ruling!(
        "Rohirrim Chargers",
        "You can exert Rohirrim Chargers as you declare it as an attacking creature. You can't do so later in combat, and creatures put onto the battlefield attacking can't be exerted. Any abilities that trigger on exerting an attacking creature will resolve before blockers are declared."
    );
    has_ability("Rohirrim Chargers", "exert");
    only_unsupported("Rohirrim Chargers", "reveal cards from the top of your library until");
    supported("Trueheart Twins");
    // Rohirrim Chargers (4/4): "You may exert this creature as it attacks." Trueheart Twins:
    // "Whenever you exert a creature, creatures you control get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let rc = t.battlefield(P0, "Rohirrim Chargers");
    t.battlefield(P0, "Trueheart Twins");
    t.battlefield(P1, "Grizzly Bears");
    let is_blocks = |d: &Decision| matches!(d, Decision::DeclareBlockers { .. });
    let seen = watch(&mut t, P1, is_blocks, chargers_power);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.attack(&[(rc, Entity::Player(P1))], &[]);
    assert!(exerted(&t, rc));
    // The exert trigger resolved before blockers were declared.
    assert_eq!(*seen.lock().unwrap(), vec![5]);
    assert_eq!(count_asked(&t, from, is_exert_question), 1);
    // Declined as it's declared: not asked again later in combat.
    let mut t = TestGame::new(2);
    let rc = t.battlefield(P0, "Rohirrim Chargers");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.attack(&[(rc, Entity::Player(P1))], &[]);
    assert!(!exerted(&t, rc));
    assert_eq!(count_asked(&t, from, is_exert_question), 1);
    // Put onto the battlefield attacking: it can't be exerted.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let from = t.asked().len();
    let card = t.exile(P0, "Rohirrim Chargers");
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(P0),
                attacking: Some(Entity::Player(P1)),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    t.g.flush_events();
    t.settle();
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(count_asked(&t, from, is_exert_question), 0);
    assert!(!exerted(&t, new));
}

#[test]
fn tap_and_freeze_and_stun_counters_dont_exert_rohirrim_chargers() {
    cr!("701.43a", "122.1d");
    ruling!(
        "Rohirrim Chargers",
        "You can't exert a creature unless an effect allows you to do so. Similar effects that \"tap and freeze\" a creature or put stun counters on a creature don't exert that creature."
    );
    has_ability("Rohirrim Chargers", "exert");
    only_unsupported("Rohirrim Chargers", "reveal cards from the top of your library until");
    supported("Decision Paralysis");
    // Trueheart Twins would see an exert.
    let mut t = TestGame::new(2);
    let twins = t.battlefield(P0, "Trueheart Twins");
    let rc = t.battlefield(P0, "Rohirrim Chargers");
    let other = t.battlefield(P0, "Rohirrim Chargers");
    // Tap and freeze.
    t.lands(P1, "Island", 4);
    let dp = t.hand(P1, "Decision Paralysis");
    t.answer_targets(P1, &[Entity::Object(rc)]);
    t.cast(P1, dp).go();
    t.resolve_all();
    assert!(tapped(&t, rc));
    // Stun counters.
    t.g.tap(other);
    t.g.add_counters(Entity::Object(other), counters::STUN, 1, None);
    t.g.flush_events();
    t.settle();
    assert!(!exerted(&t, rc));
    assert!(!exerted(&t, other));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.pt(twins), (4, 4));
    // Both stay tapped through P0's next untap step anyway (the stun counter is removed
    // instead of untapping).
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, rc));
    assert!(tapped(&t, other));
    assert_eq!(t.counters(other, counters::STUN), 0);
}

#[test]
fn the_tarrasque_must_fight_if_there_is_a_legal_target() {
    cr!("603.3d", "701.14a");
    ruling!(
        "The Tarrasque",
        "Fighting is not optional. If there is at least one legal target for The Tarrasque's last ability, it must fight."
    );
    has_ability("The Tarrasque", "it fights target creature defending player controls");
    // (Its "has haste and ward {10} as long as it was cast" compiles too: see
    // `tests/cards/grant_conditions.rs`.)
    assert!(card("The Tarrasque").unsupported_text().is_empty());
    // "Whenever The Tarrasque attacks, it fights target creature defending player
    // controls." P0 tries to choose no target: one is chosen anyway.
    let mut t = TestGame::new(2);
    let tarrasque = t.battlefield(P0, "The Tarrasque");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    attack_with(&mut t, &[(tarrasque, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(damage_on(&t, tarrasque), 2);
}

#[test]
fn domri_leaving_before_its_fight_resolves_loses_the_bonus() {
    cr!("701.14a", "611.3a", "704.5i");
    ruling!(
        "Domri, Anarch of Bolas",
        "If Domri leaves the battlefield before his last ability resolves, most likely because he only had 2 loyalty when you activated the ability, the creature won't have +1/+0 from Domri's static ability while it fights."
    );
    has_ability("Domri, Anarch of Bolas", "Creatures you control get +1/+0.");
    has_ability(
        "Domri, Anarch of Bolas",
        "Target creature you control fights target creature you don't control.",
    );
    // Its "+1: Add {R} or {G}. Creature spells you cast this turn can't be countered." is
    // compiled too now (see `tests/cards/restriction_grammar_rules.rs`).
    supported("Domri, Anarch of Bolas");
    // Domri: "Creatures you control get +1/+0." and "−2: Target creature you control
    // fights target creature you don't control." Grizzly Bears (3/2 with Domri) fight
    // Hill Giant (3/3).
    for loyalty in [2u32, 3] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let domri = t.battlefield(P0, "Domri, Anarch of Bolas");
        let now = t.counters(domri, counters::LOYALTY);
        t.g.remove_counters(Entity::Object(domri), counters::LOYALTY, now);
        t.g.add_counters(Entity::Object(domri), counters::LOYALTY, loyalty, None);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.g.recompute();
        assert_eq!(t.pt(bears), (3, 2));
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.answer_targets(P0, &[Entity::Object(giant)]);
        activate_containing(&mut t, P0, domri, "fights").expect("activate");
        t.settle();
        assert_eq!(t.on_battlefield(domri), loyalty > 2);
        t.resolve_all();
        if loyalty == 2 {
            // Domri was gone: the Bears dealt only 2.
            assert!(t.on_battlefield(giant));
            assert_eq!(damage_on(&t, giant), 2);
        } else {
            assert!(!t.on_battlefield(giant));
        }
    }
}

#[test]
fn strax_dies_before_glory_of_battle_can_save_it() {
    cr!("704.3", "704.5g", "603.3");
    ruling!(
        "Strax, Sontaran Nurse",
        "If Strax is dealt damage equal to its toughness while fighting, it will die before the +1/+1 counter from its last ability can save it."
    );
    has_ability(
        "Strax, Sontaran Nurse",
        "Whenever ~ deals damage to a creature, put a +1/+1 counter on ~.",
    );
    // (Grenades!, once unsupported, now compiles: see `choice_grammar_random`.)
    supported("Strax, Sontaran Nurse");
    supported("Prey Upon");
    supported("Feral Krushok");
    // Strax (5/5): "Glory of Battle — Whenever Strax deals damage to a creature, put a
    // +1/+1 counter on Strax." It fights Feral Krushok (5/4) via Prey Upon.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let strax = t.battlefield(P0, "Strax, Sontaran Nurse");
    let krushok = t.battlefield(P1, "Feral Krushok");
    t.lands(P0, "Forest", 1);
    let pu = t.hand(P0, "Prey Upon");
    t.cast(P0, pu).target(strax).target(krushok).go();
    t.resolve();
    // State-based actions: both died before Glory of Battle's trigger resolved.
    assert!(!t.on_battlefield(strax));
    assert!(!t.on_battlefield(krushok));
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Strax, Sontaran Nurse"));
}

