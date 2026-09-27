//! Rulings batch S10 — islandwalk (CR 702.14): "This creature can't be blocked as long
//! as defending player controls an Island." With Stormtide Leviathan: "Islandwalk / All
//! lands are Islands in addition to their other types. / Creatures without flying or
//! islandwalk can't attack."

use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::can_attack;
use crate::r_s05_common::{enter, run_from};
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `blocker` could block `attacker` now.
fn can_block(t: &TestGame, blocker: ObjectId, attacker: ObjectId) -> bool {
    t.g.can_block(t.g.current(blocker), t.g.current(attacker))
}

#[test]
fn islandwalk_cares_about_the_land_type_island() {
    cr!("702.14c", "305.6");
    ruling!(
        "Stormtide Leviathan",
        "Islandwalk cares about lands with the land type Island, not necessarily lands named Island."
    );
    supported("Stormtide Leviathan");
    // P1 controls only a Forest, which Stormtide Leviathan makes an Island too: P1's
    // creatures can't block the Leviathan.
    let mut t = TestGame::new(2);
    let leviathan = t.battlefield(P0, "Stormtide Leviathan");
    let forest = t.battlefield(P1, "Forest");
    let wall = t.battlefield(P1, "Wall of Wood");
    assert_eq!(t.obj_now(forest).chars.name, "Forest");
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    attack_with(&mut t, &[(leviathan, Entity::Player(P1))]);
    assert!(!can_block(&t, wall, leviathan));
    // A creature with islandwalk the defending player controls no land of: blockable.
    // Here a land that isn't named Island but is one (Watery Grave, Island Swamp) is
    // enough to make an islandwalker unblockable.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword {
                text: Some("Islandwalk".into()),
                filter: Some(Filter::Subtype("Island".into())),
                ..Keyword::new(KeywordKind::Landwalk)
            })],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    let wall = t.battlefield(P1, "Wall of Wood");
    t.battlefield(P1, "Swamp");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(can_block(&t, wall, bears));
    t.battlefield(P1, "Watery Grave");
    t.g.recompute();
    assert!(!can_block(&t, wall, bears));
}

#[test]
fn stormtide_leviathan_makes_every_land_an_island_in_addition() {
    cr!("305.6", "613.1d");
    ruling!(
        "Stormtide Leviathan",
        "Stormtide Leviathan's second ability causes each land on the battlefield to have the land type Island. Each land thus has the ability \"{T}: Add {U}.\" Nothing else changes about those lands"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stormtide Leviathan");
    let forest = t.battlefield(P1, "Forest");
    let o = t.obj_now(forest);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.chars.has_subtype("Forest") && o.chars.has_subtype("Island"));
    assert!(o.chars.has_supertype(Supertype::Basic));
    // Its mana abilities: {T}: Add {G} and {T}: Add {U}.
    let mana: Vec<_> = o
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.starts_with("{T}: Add"))
        .map(|a| a.text.to_string())
        .collect();
    assert!(mana.iter().any(|m| m.contains("{G}")));
    assert!(mana.iter().any(|m| m.contains("{U}")));
}

#[test]
fn creatures_without_flying_or_islandwalk_cant_attack() {
    cr!("508.1c", "702.14c");
    ruling!(
        "Stormtide Leviathan",
        "Stormtide Leviathan's third ability affects all creatures with neither flying nor islandwalk, regardless of who controls them. They can't attack any player or planeswalker."
    );
    let mut t = TestGame::new(2);
    let leviathan = t.battlefield(P0, "Stormtide Leviathan");
    let own_bears = t.battlefield(P0, "Grizzly Bears");
    let bird = t.battlefield(P0, "Storm Crow");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, leviathan));
    assert!(can_attack(&mut t, bird));
    assert!(!can_attack(&mut t, own_bears));
    // The opponent's creatures too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stormtide Leviathan");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let walker = t.battlefield(P1, "Grizzly Bears");
    run_from(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword {
                text: Some("Islandwalk".into()),
                filter: Some(Filter::Subtype("Island".into())),
                ..Keyword::new(KeywordKind::Landwalk)
            })],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(walker)],
    );
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
    assert!(can_attack(&mut t, walker));
}

#[test]
fn stormtide_leviathan_losing_its_abilities_still_makes_lands_islands() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "Stormtide Leviathan",
        "If Stormtide Leviathan loses its abilities, all lands on the battlefield (including those that enter the battlefield later on) will still be Islands in addition to their other types"
    );
    supported("Merfolk Trickster");
    // Merfolk Trickster: "tap target creature an opponent controls. It loses all
    // abilities until end of turn."
    let mut t = TestGame::new(2);
    let leviathan = t.battlefield(P0, "Stormtide Leviathan");
    t.answer_targets(P1, &[Entity::Object(leviathan)]);
    enter(&mut t, P1, "Merfolk Trickster");
    t.resolve_all();
    assert!(!t.obj_now(leviathan).has_keyword(KeywordKind::Landwalk));
    let forest = t.battlefield(P1, "Forest");
    let later = enter(&mut t, P1, "Mountain");
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    assert!(t.obj_now(later).chars.has_subtype("Island"));
}
