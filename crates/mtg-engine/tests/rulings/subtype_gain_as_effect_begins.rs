//! A subtype a resolving spell or ability gives an object "in addition to its other types"
//! is gained only if it corresponds to one of the object's types as the effect begins
//! (CR 205.3d, 611.2c; register
//! `subtype-gained-only-if-it-corresponds-as-the-effect-begins`, decided by the user Nov 19).
//! One the object could gain then follows 205.3d continuously (Terror of Towashi's ruling).
//! See also `r_residue_type_change_layers.rs`
//! (`shadow_puppeteers_on_a_creature_that_stopped_being_a_creature`) and `r_p129_misc.rs`
//! (`terror_of_towashi_makes_a_lasting_phyrexian_only_while_a_creature`).

use crate::r_residue_type_change_common::{
    attack_with, cast_new, creature_types, has_color, is_type, obj, pt_now, puppeteers_attack,
    subtype, supported,
};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Shadow Puppeteers' Dragon on a creature that is one as the ability resolves: the effect
/// gives it the Dragon type, which it loses (with its other creature types) while Swift
/// Reconfiguration makes it a noncreature Vehicle (CR 205.3d, 205.1a); the color stays.
/// (Whether crewing it brings its creature types back is a separate question of 205.1a's
/// "for the entire time the object's card type is removed": `docs/agents/items/
/// rulings-residue.md`. Terror of Towashi's ruling, a creature type coming back once the
/// object is a creature again, is tested in `r_p129_misc.rs`.)
#[test]
fn shadow_puppeteers_dragon_of_a_creature_goes_away_when_it_stops_being_one() {
    cr!("205.3d", "611.2c", "613.1d");
    interp!("subtype-gained-only-if-it-corresponds-as-the-effect-begins");
    supported("Shadow Puppeteers");
    supported("Swift Reconfiguration");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let crow = t.battlefield(P0, "Storm Crow");
    puppeteers_attack(&mut t, crow);
    t.resolve_all();
    assert!(subtype(&mut t, crow, "Dragon"));
    assert_eq!(pt_now(&mut t, crow), (4, 4));
    // Swift Reconfiguration (flash): a Vehicle artifact, not a creature, so not a Dragon.
    cast_new(&mut t, P0, "Swift Reconfiguration", &[obj(crow)]);
    t.resolve();
    assert!(!is_type(&mut t, crow, CardType::Creature));
    assert!(!subtype(&mut t, crow, "Dragon"));
    assert!(subtype(&mut t, crow, "Vehicle"));
    assert!(has_color(&mut t, crow, Color::Red));
    assert!(has_color(&mut t, crow, Color::Blue));
}

/// Terror of Towashi returning Heliod, God of the Sun while P0's devotion to white is below
/// five: Heliod isn't a creature as the effect begins, so it never becomes a Phyrexian, even
/// once it's a creature. (Terror of Towashi's ruling is about a creature that "stops being a
/// creature after the ability has resolved": `r_p129_misc.rs`.)
#[test]
fn terror_of_towashi_never_makes_a_noncreature_a_phyrexian() {
    cr!("205.3d", "611.2c");
    interp!("subtype-gained-only-if-it-corresponds-as-the-effect-begins");
    supported("Terror of Towashi");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let terror = t.battlefield(P0, "Terror of Towashi");
    let god = t.graveyard(P0, "Heliod, God of the Sun");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(god)]);
    attack_with(&mut t, &[(terror, Entity::Player(P1))]);
    t.resolve_all();
    let god = t.g.current(god);
    assert!(t.on_battlefield(god));
    assert!(!is_type(&mut t, god, CardType::Creature));
    assert!(!subtype(&mut t, god, "Phyrexian"));
    // Devotion to white five: Heliod is a creature, but not a Phyrexian.
    for _ in 0..4 {
        t.battlefield(P0, "Benalish Knight");
    }
    assert!(is_type(&mut t, god, CardType::Creature));
    assert!(!subtype(&mut t, god, "Phyrexian"));
}

/// A characteristic rider on a permanent put onto the battlefield ("That creature is a black
/// Zombie in addition to its other colors and types", an effect that applies as it enters,
/// CR 611.2e) follows the same rule: Heliod, God of the Sun returned while P0's devotion to
/// white is below five isn't a creature as the effect begins, so it never becomes a Zombie,
/// even once it's a creature. It's black, in addition to white.
#[test]
fn rise_from_the_grave_never_makes_a_noncreature_a_zombie() {
    cr!("205.3d", "611.2c", "611.2e", "205.1b");
    interp!("subtype-gained-only-if-it-corresponds-as-the-effect-begins");
    supported("Rise from the Grave");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let god = t.graveyard(P0, "Heliod, God of the Sun");
    cast_new(&mut t, P0, "Rise from the Grave", &[obj(god)]);
    t.resolve();
    let god = t.g.current(god);
    assert!(t.on_battlefield(god));
    assert!(!is_type(&mut t, god, CardType::Creature));
    assert!(!subtype(&mut t, god, "Zombie"));
    assert!(has_color(&mut t, god, Color::Black) && has_color(&mut t, god, Color::White));
    // Devotion to white five: Heliod is a creature, but not a Zombie.
    for _ in 0..4 {
        t.battlefield(P0, "Benalish Knight");
    }
    assert!(is_type(&mut t, god, CardType::Creature));
    assert!(!subtype(&mut t, god, "Zombie"));
    assert!(subtype(&mut t, god, "God"));
    assert!(has_color(&mut t, god, Color::Black));
}

/// An effect that makes a noncreature a creature and gives it a creature type at once gives
/// it the type: the card types the effect itself adds count as it begins. (Mishra's Factory
/// grants no keyword, so nothing else has computed its characteristics with the new effect.)
#[test]
fn an_effect_making_a_noncreature_a_creature_of_a_type_gives_it_the_type() {
    cr!("205.3d", "205.1b");
    interp!("subtype-gained-only-if-it-corresponds-as-the-effect-begins");
    supported("Elemental Uprising");
    supported("Mishra's Factory");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let land = t.battlefield(P0, "Wastes");
    // "Target land you control becomes a 4/4 Elemental creature with haste until end of
    // turn. It's still a land."
    cast_new(&mut t, P0, "Elemental Uprising", &[obj(land)]);
    t.resolve();
    assert!(is_type(&mut t, land, CardType::Creature));
    assert!(is_type(&mut t, land, CardType::Land));
    assert_eq!(creature_types(&mut t, land), vec!["Elemental".to_string()]);
    assert_eq!(pt_now(&mut t, land), (4, 4));
    // "{1}: Mishra's Factory becomes a 2/2 Assembly-Worker artifact creature until end of
    // turn. It's still a land."
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.lands(P0, "Wastes", 1);
    crate::r_s06_common::activate_containing(&mut t, P0, factory, "becomes").unwrap();
    t.resolve_all();
    assert!(is_type(&mut t, factory, CardType::Creature));
    assert!(is_type(&mut t, factory, CardType::Artifact));
    assert!(is_type(&mut t, factory, CardType::Land));
    assert_eq!(
        creature_types(&mut t, factory),
        vec!["Assembly-Worker".to_string()]
    );
    assert_eq!(pt_now(&mut t, factory), (2, 2));
}
