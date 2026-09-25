//! Triggers on players being attacked (CR 508.3a, 508.3b, 508.3e) and on noncombat
//! damage.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// Asserts that the card's abilities containing `needle` compiled.
fn assert_line_supported(name: &str, needle: &str) {
    let c = card(name);
    let bad: Vec<_> = c
        .unsupported_text()
        .into_iter()
        .filter(|u| u.contains(needle))
        .collect();
    assert!(bad.is_empty(), "{name} has unsupported text: {bad:?}");
}

#[test]
fn whenever_you_attack_a_player_triggers_once_per_player() {
    cr!("508.3e", "603.2c");
    assert_line_supported("Long-Range Sensor", "Whenever you attack a player");
    // Two creatures attacking one player: one trigger.
    let mut t = TestGame::new(2);
    let sensor = t.battlefield(P0, "Long-Range Sensor");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(sensor, "charge"), 1);
    // Attacking two players: once for each.
    let mut t = TestGame::new(3);
    let sensor = t.battlefield(P0, "Long-Range Sensor");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    assert_eq!(t.counters(sensor, "charge"), 2);
}

#[test]
fn attacking_only_a_planeswalker_isnt_attacking_a_player() {
    cr!("508.3e");
    assert_line_supported("Long-Range Sensor", "Whenever you attack a player");
    let mut t = TestGame::new(2);
    let sensor = t.battlefield(P0, "Long-Range Sensor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Object(pw))], &[]);
    assert_eq!(t.counters(sensor, "charge"), 0);
}

#[test]
fn opponent_attacks_you_with_two_or_more_creatures() {
    cr!("508.3e");
    assert_line_supported("Everett K. Ross, Hapless Attaché", "two or more creatures");
    for n in [1usize, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Everett K. Ross, Hapless Attaché");
        let attackers: Vec<_> = (0..n)
            .map(|_| (t.battlefield(P1, "Grizzly Bears"), Entity::Player(P0)))
            .collect();
        let hand = t.hand_size(P0);
        t.set_step(P1, Step::BeginningOfCombat);
        t.attack(&attackers, &[]);
        let drew = if n >= 2 { 1 } else { 0 };
        assert_eq!(t.hand_size(P0), hand + drew, "{n} attackers");
    }
}

#[test]
fn a_creature_attacks_enchanted_player() {
    cr!("508.3a", "303.4b");
    assert_supported(&["Curse of Predation"]);
    let mut t = TestGame::new(3);
    let curse = t.battlefield(P0, "Curse of Predation");
    t.g.attach(curse, Entity::Player(P1));
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // Only the creature attacking the enchanted player gets a counter.
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 0);
}

#[test]
fn opponent_is_dealt_noncombat_damage() {
    cr!("120.3a", "510.2");
    assert_supported(&["Chandra's Spitfire"]);
    let mut t = TestGame::new(2);
    let spitfire = t.battlefield(P0, "Chandra's Spitfire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Combat damage doesn't count.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.pt(spitfire), (1, 3));
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(spitfire), (4, 3));
}

#[test]
fn is_dealt_noncombat_damage_that_many_treasures() {
    cr!("120.3", "603.2");
    assert_line_supported("Smaug the Impenetrable", "noncombat damage");
    let mut t = TestGame::new(2);
    let smaug = t.battlefield(P0, "Smaug the Impenetrable");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(smaug).go();
    t.resolve_all();
    let treasures =
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Treasure"))
            .count();
    assert_eq!(treasures, 3);
}

#[test]
fn shortened_name_refers_to_the_card() {
    cr!("201.5c", "508.3a");
    // "Whenever Edgar attacks, put a +1/+1 counter on each Vampire you control."
    assert_line_supported("Edgar Markov", "attacks");
    let mut t = TestGame::new(2);
    let edgar = t.battlefield(P0, "Edgar Markov");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // Another creature attacking doesn't trigger it.
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(edgar, "+1/+1"), 0);
    let mut t = TestGame::new(2);
    let edgar = t.battlefield(P0, "Edgar Markov");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(edgar, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(edgar, "+1/+1"), 1);
}
