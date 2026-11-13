//! Type-, color- and P/T-setting statics (CR 205.1, 305.7, 613.1d-e, 613.4b) and
//! restrictions on the affected objects (CR 508.1c-d, 509.1b-c), compiled by
//! `oracle/patterns/statics.rs`.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The kinds of mana a permanent's own mana abilities can produce.
fn produces(t: &TestGame, id: ObjectId) -> Vec<ManaType> {
    use mtg_engine::ability::*;
    let mut out = Vec::new();
    for a in &t.obj_now(id).chars.abilities {
        if let AbilityKind::Activated(act) = &a.kind {
            if let Effect::AddMana {
                mana: ManaProduction::Fixed(v),
                ..
            } = &act.body.effect
            {
                out.extend(v.iter().copied());
            }
        }
    }
    out
}

#[test]
fn is_also_adds_creature_types() {
    cr!("205.1b", "613.1d");
    compiles("Stonework Packbeast");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Stonework Packbeast");
    let c = &t.obj_now(beast).chars;
    for s in ["Beast", "Cleric", "Rogue", "Warrior", "Wizard"] {
        assert!(c.has_subtype(s), "missing {s}");
    }
}

#[test]
fn enchanted_land_is_an_island() {
    cr!("305.7", "305.6", "613.1d");
    compiles("Spreading Seas");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let seas = t.battlefield(P0, "Spreading Seas");
    t.attach(seas, Entity::Object(forest));
    t.settle();
    let c = &t.obj_now(forest).chars;
    assert!(c.has_subtype("Island"));
    assert!(!c.has_subtype("Forest"));
    assert_eq!(produces(&t, forest), vec![ManaType::U]);
}

#[test]
fn nonbasic_lands_are_mountains() {
    cr!("305.7", "613.1d");
    ruling!(
        "Blood Moon",
        "They will gain the land type Mountain and gain the ability"
    );
    compiles("Blood Moon");
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P1, "Mishra's Factory");
    let forest = t.battlefield(P1, "Forest");
    assert_eq!(produces(&t, tomb), vec![ManaType::C]);
    assert_eq!(t.obj_now(tomb).chars.abilities.len(), 3);
    t.battlefield(P0, "Blood Moon");
    // The nonbasic land loses its rules-text abilities and taps for {R}.
    assert!(t.obj_now(tomb).chars.has_subtype("Mountain"));
    assert_eq!(produces(&t, tomb), vec![ManaType::R]);
    assert_eq!(t.obj_now(tomb).chars.abilities.len(), 1);
    // It's still a land, and keeps its name; basics are unaffected.
    assert!(t.obj_now(tomb).is(CardType::Land));
    assert_eq!(t.obj_now(tomb).chars.name.as_str(), "Mishra's Factory");
    assert_eq!(produces(&t, forest), vec![ManaType::G]);
}

#[test]
fn lands_become_every_basic_land_type() {
    cr!("305.7", "205.1b");
    compiles("Prismatic Omen");
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P0, "Mishra's Factory");
    let theirs = t.battlefield(P1, "Mishra's Factory");
    t.battlefield(P0, "Prismatic Omen");
    // "In addition to their other types": it keeps its own ability and gains five.
    let mut kinds = produces(&t, tomb);
    kinds.sort_by_key(|k| format!("{k:?}"));
    assert_eq!(kinds.len(), 6);
    for s in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(t.obj_now(tomb).chars.has_subtype(s));
    }
    assert!(!t.obj_now(theirs).chars.has_subtype("Island"));
}

#[test]
fn becomes_a_blue_frog_with_base_pt() {
    cr!("205.1a", "613.1d", "613.1e", "613.1f", "613.4b");
    ruling!("Frogify", "It's just a blue Frog");
    compiles("Frogify");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let frogify = t.battlefield(P0, "Frogify");
    t.attach(frogify, Entity::Object(angel));
    t.settle();
    let o = t.obj_now(angel);
    assert_eq!(t.pt(angel), (1, 1));
    assert!(!o.has_keyword(KeywordKind::Flying));
    assert!(!o.has_keyword(KeywordKind::Vigilance));
    assert!(o.chars.has_subtype("Frog"));
    assert!(!o.chars.has_subtype("Angel"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    // P/T modifications still apply on top of the new base (layer 7c after 7b).
    let strength = t.battlefield(P1, "Holy Strength");
    t.attach(strength, Entity::Object(angel));
    t.settle();
    assert_eq!(t.pt(angel), (2, 3));
}

#[test]
fn equipment_makes_a_black_zombie() {
    cr!("205.1a", "613.1d", "613.1e");
    ruling!("Nim Deathmantle", "that creature will be a black Zombie");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mantle = t.battlefield(P0, "Nim Deathmantle");
    t.attach(mantle, Entity::Object(bears));
    t.settle();
    let o = t.obj_now(bears);
    assert_eq!(t.pt(bears), (4, 4));
    assert!(o.has_keyword(KeywordKind::Intimidate));
    assert!(o.chars.has_subtype("Zombie"));
    assert!(!o.chars.has_subtype("Bear"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
}

#[test]
fn enchanted_artifact_becomes_a_five_five_creature() {
    cr!("205.1b", "613.1d", "613.4b");
    compiles("Ensoul Artifact");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let ensoul = t.battlefield(P0, "Ensoul Artifact");
    t.attach(ensoul, Entity::Object(ring));
    t.settle();
    let o = t.obj_now(ring);
    assert!(o.is_creature());
    assert!(o.is(CardType::Artifact));
    assert_eq!(t.pt(ring), (5, 5));
}

#[test]
fn all_lands_are_creatures_that_are_still_lands() {
    cr!("205.1b", "613.1d", "613.4b");
    compiles("Nature's Revolt");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.battlefield(P0, "Nature's Revolt");
    let o = t.obj_now(forest);
    assert!(o.is_creature());
    assert!(o.is(CardType::Land));
    assert!(o.chars.has_subtype("Forest"));
    assert_eq!(t.pt(forest), (2, 2));
}

#[test]
fn other_creatures_have_base_power_and_toughness() {
    cr!("613.4b", "613.4c");
    compiles("Godhead of Awe");
    let mut t = TestGame::new(2);
    let godhead = t.battlefield(P0, "Godhead of Awe");
    let angel = t.battlefield(P1, "Serra Angel");
    assert_eq!(t.pt(godhead), (4, 4));
    assert_eq!(t.pt(angel), (1, 1));
    // A +1/+2 Aura applies after the base P/T is set.
    let strength = t.battlefield(P1, "Holy Strength");
    t.attach(strength, Entity::Object(angel));
    t.settle();
    assert_eq!(t.pt(angel), (2, 3));
}

// ---------------------------------------------------------------------------
// Restrictions and requirements
// ---------------------------------------------------------------------------

#[test]
fn enchanted_creature_gets_minus_two_and_cant_block() {
    cr!("509.1b", "613.4c");
    compiles("Cast into Darkness");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = t.battlefield(P0, "Cast into Darkness");
    t.attach(aura, Entity::Object(giant));
    t.settle();
    assert_eq!(t.pt(giant), (1, 3));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // The declared block is illegal; the Bears deals damage to the player.
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn creatures_without_flying_cant_attack() {
    cr!("508.1c");
    compiles("Moat");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Moat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.can_attack(bears));
    assert!(t.can_attack(angel));
}

#[test]
fn all_creatures_attack_each_combat_if_able() {
    cr!("508.1d");
    compiles("Grand Melee");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grand Melee");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // The attacking player declares no attackers; the requirement adds the Bears.
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn islands_dont_untap() {
    cr!("502.3");
    compiles("Choke");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Choke");
    let island = t.battlefield(P1, "Island");
    let forest = t.battlefield(P1, "Forest");
    t.tap(island);
    t.tap(forest);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(island).tapped);
    assert!(!t.obj_now(forest).tapped);
}
