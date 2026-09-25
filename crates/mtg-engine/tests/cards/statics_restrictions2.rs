//! More rule-modifying statics: untap limits (CR 502.3), base toughness, attacks and
//! blocks limited by power, combined restrictions, and conditions joined by "and"/"or".

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn untapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| !t.obj_now(**id).tapped).count()
}

#[test]
fn players_cant_untap_more_than_one_land() {
    cr!("502.3", "611.3a");
    ruling!(
        "Winter Orb",
        "If Winter Orb is tapped as your untap step begins, your lands will all untap."
    );
    compiles("Winter Orb");
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P1, "Winter Orb");
    let lands = t.lands(P0, "Forest", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for id in lands.iter().chain([&bears]) {
        t.g.tap(*id);
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(untapped(&t, &lands), 1);
    // Other permanents untap as usual.
    assert!(!t.obj_now(bears).tapped);
    // With the Orb tapped, everything untaps.
    for id in &lands {
        t.g.tap(*id);
    }
    t.g.tap(orb);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(untapped(&t, &lands), 3);
}

#[test]
fn untap_limits_dont_add_together() {
    cr!("502.3");
    ruling!("Static Orb", "they do not add together");
    compiles("Static Orb");
    compiles("Smoke");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Static Orb");
    t.battlefield(P1, "Static Orb");
    t.battlefield(P1, "Smoke");
    let lands = t.lands(P0, "Forest", 2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let all = [lands[0], lands[1], a, b];
    for id in &all {
        t.g.tap(*id);
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    // Two permanents in all, at most one of them a creature.
    assert_eq!(untapped(&t, &all), 2);
    assert!(untapped(&t, &[a, b]) <= 1);
}

#[test]
fn keyword_list_with_protection_from_two_colors() {
    cr!("613.1f", "702.16a");
    compiles("Akroma's Memorial");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akroma's Memorial");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for k in [
        KeywordKind::Flying,
        KeywordKind::FirstStrike,
        KeywordKind::Vigilance,
        KeywordKind::Trample,
        KeywordKind::Haste,
        KeywordKind::Protection,
    ] {
        assert!(has(&t, bears, k), "{k:?}");
    }
    // Protection from black and from red: a red spell can't target it.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    let _ = t.cast(P1, bolt).target(bears).try_go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn creatures_your_opponents_control_have_base_toughness_one() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Maha, Its Feathers Night",
        "Effects that modify a creature’s power and/or toughness"
    );
    compiles("Maha, Its Feathers Night");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maha, Its Feathers Night");
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = t.battlefield(P1, "Arcane Flight");
    t.attach(aura, Entity::Object(giant));
    t.settle();
    // Base 3/1, then +1/+1.
    assert_eq!(t.pt(giant), (4, 2));
}

#[test]
fn creatures_with_power_greater_than_cards_in_hand_cant_attack() {
    cr!("508.1c");
    ruling!(
        "Ensnaring Bridge",
        "checks the number of cards in your hand only while a player (including you) is declaring attackers"
    );
    compiles("Ensnaring Bridge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ensnaring Bridge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.hand(P0, "Island");
    t.set_step(P1, Step::BeginningOfCombat);
    // One card in the Bridge controller's hand.
    assert!(!t.can_attack(bears));
    assert!(t.can_attack(elves));
    t.hand(P0, "Island");
    t.g.recompute();
    assert!(t.can_attack(bears));
}

#[test]
fn creatures_with_power_less_than_a_count_cant_block_it() {
    cr!("509.1b");
    compiles("Kraken of the Straits");
    for (islands, damage) in [(3, 6), (2, 0)] {
        let mut t = TestGame::new(2);
        let kraken = t.battlefield(P0, "Kraken of the Straits");
        t.lands(P0, "Island", islands);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(kraken, Entity::Player(P1))], &[(bears, kraken)]);
        assert_eq!(t.life(P1), 20 - damage, "{islands} Islands");
    }
}

#[test]
fn cant_attack_you_or_block_creatures_you_control() {
    cr!("508.1c", "509.1b");
    compiles("Storm, Windrider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm, Windrider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(angel, bears)]);
    assert_eq!(t.life(P1), 18);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!t.can_attack_target(angel, Entity::Player(P0)));
}

#[test]
fn without_flying_or_reach_cant_block_small_creatures() {
    cr!("509.1b");
    compiles("Sidar Kondo of Jamuraa");
    for (blocker, damage) in [("Hill Giant", 2), ("Giant Spider", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sidar Kondo of Jamuraa");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, blocker);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(bears, Entity::Player(P1))], &[(b, bears)]);
        assert_eq!(t.life(P1), 20 - damage, "{blocker}");
    }
}

#[test]
fn creatures_blocking_or_blocked_by_it_have_lifelink() {
    cr!("613.1f", "702.15b");
    compiles("Alms Beast");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Alms Beast");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!has(&t, bears, KeywordKind::Lifelink));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(beast, Entity::Player(P1))], &[(bears, beast)]);
    // The blocking Bears had lifelink as it dealt damage.
    assert_eq!(t.life(P1), 22);
}

#[test]
fn conditions_joined_by_and_or_or() {
    cr!("611.3a");
    ruling!(
        "Sidewinder Naga",
        "Controlling one is the same as controlling five"
    );
    compiles("Sidewinder Naga");
    compiles("Grond, the Gatebreaker");
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Sidewinder Naga");
    assert_eq!(t.pt(naga), (3, 2));
    t.graveyard(P0, "Desert");
    t.settle();
    assert_eq!(t.pt(naga), (4, 2));
    assert!(has(&t, naga, KeywordKind::Trample));
    // "As long as it's your turn and you control an Army"
    let grond = t.battlefield(P0, "Grond, the Gatebreaker");
    t.set_step(P0, Step::PrecombatMain);
    assert!(!t.obj_now(grond).is(CardType::Creature));
    let army = t.battlefield(P0, "Grizzly Bears");
    // An Army (as amass would create).
    t.g.objects[army.0 as usize].base.subtypes.push("Army".into());
    t.g.dirty = true;
    t.settle();
    assert!(t.obj_now(grond).is(CardType::Creature));
    t.set_step(P1, Step::PrecombatMain);
    t.g.dirty = true;
    t.settle();
    assert!(!t.obj_now(grond).is(CardType::Creature));
}
