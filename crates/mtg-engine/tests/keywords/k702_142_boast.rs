//! CR 702.142 Boast.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::run_effect;
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Attacks with `attackers` (P0's turn) and finishes combat without blocks.
fn attack(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&decl, &[]);
}

#[test]
fn a_boast_ability_can_be_activated_only_if_the_creature_attacked_this_turn() {
    cr!("702.142", "702.142a");
    assert_supported("Axgard Braggart");
    let mut t = TestGame::new(2);
    // Axgard Braggart: "Boast — {1}{W}: Untap this creature. Put a +1/+1 counter on it."
    let braggart = t.battlefield(P0, "Axgard Braggart");
    let boast = ability_uid(&mut t, braggart, "Boast");
    add_mana(&mut t, P0, ManaType::W, 2);
    // Not before it attacked.
    assert!(!activatable(&mut t, P0, braggart, boast));
    attack(&mut t, &[braggart]);
    // After combat, in the same turn: it attacked this turn.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.obj(braggart).tapped);
    add_mana(&mut t, P0, ManaType::W, 4);
    assert!(activatable(&mut t, P0, braggart, boast));
    activate_uid(&mut t, P0, braggart, boast).unwrap();
    t.resolve_all();
    assert!(!t.obj(braggart).tapped);
    assert_eq!(t.counters(braggart, "+1/+1"), 1);
    // Only once each turn.
    assert!(!activatable(&mut t, P0, braggart, boast));
    assert!(activate_uid(&mut t, P0, braggart, boast).is_err());
    // Next turn it hasn't attacked (yet).
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::W, 2);
    assert!(!activatable(&mut t, P0, braggart, boast));
}

#[test]
fn a_boast_ability_can_be_activated_any_time_after_it_was_declared_as_an_attacker() {
    cr!("702.142a");
    ruling!(
        "Dragonkin Berserker",
        "A boast ability can be activated at any point after the creature with that ability has been declared as an attacker."
    );
    ruling!(
        "Axgard Braggart",
        "You can activate Axgard Braggart's boast ability even if Axgard Braggart is already untapped. You'll still put a +1/+1 counter on it."
    );
    let mut t = TestGame::new(2);
    let braggart = t.battlefield(P0, "Axgard Braggart");
    let boast = ability_uid(&mut t, braggart, "Boast");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(braggart, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::W, 2);
    // During the declare attackers step, before blockers are declared.
    activate_uid(&mut t, P0, braggart, boast).unwrap();
    t.resolve_all();
    assert!(!t.obj(braggart).tapped);
    assert_eq!(t.counters(braggart, "+1/+1"), 1);
    // It's still attacking.
    assert!(t.g.is_attacking(braggart));
}

#[test]
fn a_creature_put_onto_the_battlefield_attacking_cant_boast() {
    cr!("702.142a");
    ruling!(
        "Dragonkin Berserker",
        "If a creature with a boast ability is put onto the battlefield attacking, it was never declared as an attacker. Its boast ability can't be activated that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(bears, Entity::Player(P1))]);
    let card = t.hand(P0, "Axgard Braggart");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                attacking: true,
                ..Destination::battlefield()
            },
        },
        &[Entity::Object(card)],
    );
    let braggart = t.g.current(card);
    assert!(t.g.is_attacking(braggart));
    add_mana(&mut t, P0, ManaType::W, 2);
    let boast = ability_uid(&mut t, braggart, "Boast");
    assert!(!activatable(&mut t, P0, braggart, boast));
}

#[test]
fn a_boast_ability_is_activated_only_once_even_with_additional_combats() {
    cr!("702.142a");
    ruling!(
        "Dragonkin Berserker",
        "If an effect adds additional combat phases to a turn and a creature with a boast ability attacks more than once during that turn, its boast ability can still be activated only once."
    );
    let mut t = TestGame::new(2);
    let pup = t.battlefield(P0, "Fearless Pup");
    let boast = ability_uid(&mut t, pup, "Boast");
    attack(&mut t, &[pup]);
    add_mana(&mut t, P0, ManaType::R, 6);
    activate_uid(&mut t, P0, pup, boast).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(pup), (3, 1));
    // An additional combat phase: it attacks again (it's untapped for the test).
    t.g.objects[pup.0 as usize].tapped = false;
    attack(&mut t, &[pup]);
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(!activatable(&mut t, P0, pup, boast));
}

#[test]
fn a_player_who_gains_control_of_a_creature_that_attacked_can_boast() {
    cr!("702.142a");
    ruling!(
        "Dragonkin Berserker",
        "If it's not your turn and you gain control of a creature with a boast ability after that creature attacked, you can activate that creature's boast ability if it hasn't been activated yet that turn."
    );
    let mut t = TestGame::new(2);
    let pup = t.battlefield(P0, "Fearless Pup");
    let boast = ability_uid(&mut t, pup, "Boast");
    attack(&mut t, &[pup]);
    // P1 gains control of it during P0's turn.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(pup)],
    );
    assert_eq!(t.obj(pup).controller, P1);
    add_mana(&mut t, P1, ManaType::R, 3);
    assert!(activatable(&mut t, P1, pup, boast));
    activate_uid(&mut t, P1, pup, boast).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(pup), (3, 1));
}

#[test]
fn whenever_you_activate_a_boast_ability() {
    cr!("702.142b");
    assert_supported("Frenzied Raider");
    ruling!(
        "Frenzied Raider",
        "Frenzied Raider's triggered ability will resolve before the boast ability that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    // Frenzied Raider: "Whenever you activate a boast ability, put a +1/+1 counter on this
    // creature."
    let raider = t.battlefield(P0, "Frenzied Raider");
    let braggart = t.battlefield(P0, "Axgard Braggart");
    // Another activated ability isn't a boast ability.
    let ring = t.battlefield(P0, "Sol Ring");
    let tap = ability_uid(&mut t, ring, "{T}");
    activate_uid(&mut t, P0, ring, tap).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let boast = ability_uid(&mut t, braggart, "Boast");
    attack(&mut t, &[braggart]);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    activate_uid(&mut t, P0, braggart, boast).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger resolves first.
    t.resolve();
    assert_eq!(t.counters(raider, "+1/+1"), 1);
    assert_eq!(t.counters(braggart, "+1/+1"), 0);
    t.resolve_all();
    assert_eq!(t.counters(braggart, "+1/+1"), 1);
}

#[test]
fn boast_abilities_you_activate_cost_less() {
    cr!("702.142b");
    assert_supported("Dragonkin Berserker");
    ruling!(
        "Dragonkin Berserker",
        "The cost reduction applies only to generic mana in the activation costs of boast abilities you activate."
    );
    let mut t = TestGame::new(2);
    // Dragonkin Berserker: "Boast abilities you activate cost {1} less to activate for
    // each Dragon you control." Boast — {4}{R}: Create a 5/5 red Dragon token with flying.
    let berserker = t.battlefield(P0, "Dragonkin Berserker");
    t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Shivan Dragon");
    let boast = ability_uid(&mut t, berserker, "Boast");
    attack(&mut t, &[berserker]);
    // Two Dragons: {2}{R}.
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    activate_uid(&mut t, P0, berserker, boast).unwrap();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0).len(), 1);
    // It doesn't reduce the other activated abilities.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonkin Berserker");
    for _ in 0..5 {
        t.battlefield(P0, "Shivan Dragon");
    }
    let dragon = t.named_on_battlefield("Shivan Dragon")[0];
    // Shivan Dragon: "{R}: This creature gets +1/+0 until end of turn."
    let pump = ability_uid(&mut t, dragon, "{R}");
    assert!(!activatable(&mut t, P0, dragon, pump));
    // With five Dragons, the boast ability still costs {R}.
    let berserker = t.named_on_battlefield("Dragonkin Berserker")[0];
    let boast = ability_uid(&mut t, berserker, "Boast");
    attack(&mut t, &[berserker]);
    assert!(!activatable(&mut t, P0, berserker, boast));
    add_mana(&mut t, P0, ManaType::R, 1);
    activate_uid(&mut t, P0, berserker, boast).unwrap();
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn creatures_can_boast_twice_during_your_turn() {
    cr!("702.142b");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "You must still attack with a creature you control to activate its boast ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Birgi, God of Storytelling // Harnfel, Horn of Bounty");
    let pup = t.battlefield(P0, "Fearless Pup");
    let boast = ability_uid(&mut t, pup, "Boast");
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(!activatable(&mut t, P0, pup, boast));
    attack(&mut t, &[pup]);
    add_mana(&mut t, P0, ManaType::R, 9);
    activate_uid(&mut t, P0, pup, boast).unwrap();
    t.resolve_all();
    activate_uid(&mut t, P0, pup, boast).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(pup), (5, 1));
    assert!(!activatable(&mut t, P0, pup, boast));
}
