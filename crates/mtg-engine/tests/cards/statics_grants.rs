//! Ability-granting statics (CR 613.1f): keywords and quoted abilities given to other
//! objects, compiled by `oracle/patterns/statics.rs`.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn granted_mana_ability_pays_for_spells() {
    cr!("613.1f", "302.6");
    compiles("Cryptolith Rite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cryptolith Rite");
    let elves = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    // No lands: the Bears' granted "{T}: Add one mana of any color." pays for the Bolt.
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(t.obj_now(elves).tapped);
}

#[test]
fn granted_tap_ability_needs_haste_or_no_summoning_sickness() {
    cr!("613.1f", "302.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cryptolith Rite");
    t.battlefield_sick(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
}

#[test]
fn opponents_creatures_dont_get_you_control_grants() {
    cr!("613.1f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cryptolith Rite");
    t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
}

#[test]
fn aura_grants_an_activated_ability_this_creature_is_the_enchanted_one() {
    cr!("613.1f", "303.4e");
    compiles("Arcane Teachings");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Arcane Teachings");
    t.attach(aura, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
    let victim = t.battlefield(P1, "Savannah Lions");
    // "{T}: This creature deals 1 damage to any target." — the Bears taps and deals it.
    t.activate(P0, bears, 0, &[Entity::Object(victim)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.on_battlefield(victim));
    assert!(t.on_battlefield(aura));
}

#[test]
fn aura_on_an_opponents_creature_gives_them_the_ability() {
    cr!("613.1f", "303.4e");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Arcane Teachings");
    t.attach(aura, Entity::Object(theirs));
    t.settle();
    // Only the enchanted creature's controller can activate the granted ability.
    t.activate(P1, theirs, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn aura_grants_keywords_and_a_triggered_ability() {
    cr!("613.1f", "702.15b");
    compiles("Staggering Insight");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Staggering Insight");
    t.attach(aura, Entity::Object(bears));
    t.settle();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    // 3 damage with lifelink, and the granted trigger draws a card.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn equipment_grant_where_this_creature_is_the_equipped_creature() {
    cr!("613.1f", "301.5d");
    // Mortarpod: Equipped creature has "Sacrifice this creature: This creature deals 1
    // damage to any target."
    compiles("Mortarpod");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pod = t.battlefield(P0, "Mortarpod");
    t.attach(pod, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (2, 3));
    t.activate(P0, bears, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // The creature was sacrificed, not the Equipment.
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(pod));
}

#[test]
fn granted_ability_naming_the_card_itself_is_left_unsupported() {
    cr!("201.5a");
    // "{T}, Sacrifice Blazing Torch: Blazing Torch deals 2 damage to any target." names
    // the Equipment, not the equipped creature; it isn't compiled as if it did.
    let def = card("Blazing Torch");
    assert!(def
        .unsupported_text()
        .iter()
        .any(|u| u.starts_with("Equipped creature has")));
}

#[test]
fn enchanted_creature_loses_flying_and_gains_defender() {
    cr!("613.1f");
    compiles("Sky Tether");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let aura = t.battlefield(P0, "Sky Tether");
    t.attach(aura, Entity::Object(angel));
    t.settle();
    let o = t.obj_now(angel);
    assert!(!o.has_keyword(KeywordKind::Flying));
    assert!(o.has_keyword(KeywordKind::Defender));
    assert!(o.has_keyword(KeywordKind::Vigilance));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!t.can_attack(angel));
}

#[test]
fn keyword_grant_with_a_condition_on_the_enchanted_creature() {
    cr!("611.3a", "613.1f");
    // Hope Against Hope: +1/+1 for each creature you control; first strike as long as
    // enchanted creature is a Human.
    compiles("Hope Against Hope");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let aura = t.battlefield(P0, "Hope Against Hope");
    t.attach(aura, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    let human = t.battlefield(P0, "Elite Vanguard");
    t.attach(aura, Entity::Object(human));
    t.settle();
    assert!(t.obj_now(human).has_keyword(KeywordKind::FirstStrike));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(lions), (2, 1));
}
