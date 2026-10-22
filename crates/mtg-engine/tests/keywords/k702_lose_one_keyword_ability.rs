//! Losing one ability of a keyword that has a quality (CR 613.1f): each landwalk and each
//! protection ability is a separate ability (CR 702.14, 702.16g), so losing "islandwalk"
//! or "protection from black" leaves the others; losing all "bands with other" abilities
//! leaves plain banding (CR 702.22b). And "hexproof from activated and triggered
//! abilities", a quality of the targeting ability itself (CR 702.11d).

use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The keyword abilities of `kind` that `id` has, by their filter.
fn instances(t: &mut TestGame, id: ObjectId, kind: KeywordKind) -> Vec<Keyword> {
    t.g.recompute();
    t.g.obj(id).chars.keywords_of(kind).cloned().collect()
}

fn walks(t: &mut TestGame, id: ObjectId, land: &str) -> bool {
    instances(t, id, KeywordKind::Landwalk)
        .iter()
        .any(|k| format!("{:?}", k.filter).contains(land))
}

#[test]
fn losing_one_landwalk_ability_leaves_the_others() {
    cr!("702.14a", "113.10", "613.1f");
    let mut t = TestGame::new(2);
    let islandwalker = t.battlefield(P1, "Merfolk Raiders");
    let forestwalker = t.battlefield(P1, "Zendikar Farguide");
    let angel = t.battlefield(P1, "Serra Angel");
    assert!(walks(&mut t, islandwalker, "Island"));
    // Mystic Decree: "All creatures lose flying and islandwalk."
    t.battlefield(P0, "Mystic Decree");
    assert!(!walks(&mut t, islandwalker, "Island"));
    assert!(walks(&mut t, forestwalker, "Forest"));
    t.g.recompute();
    assert!(!t.g.obj(angel).chars.has_keyword(KeywordKind::Flying));
    assert!(t.g.obj(angel).chars.has_keyword(KeywordKind::Vigilance));
}

#[test]
fn losing_one_protection_ability_leaves_the_others() {
    cr!("702.16a", "702.16g", "613.1f");
    let mut t = TestGame::new(2);
    // Paladin en-Vec: protection from black and from red.
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    let black = t.battlefield(P0, "Drudge Skeletons");
    let red = t.battlefield(P0, "Goblin Piker");
    assert_eq!(instances(&mut t, paladin, KeywordKind::Protection).len(), 2);
    assert!(t.g.protected_from(paladin, black) && t.g.protected_from(paladin, red));
    let snitch = t.battlefield(P0, "Cephalid Snitch");
    t.activate(P0, snitch, 0, &[Entity::Object(paladin)]).unwrap();
    t.resolve_all();
    let left = instances(&mut t, paladin, KeywordKind::Protection);
    assert_eq!(left.len(), 1);
    assert!(format!("{:?}", left[0].filter).contains("Red"));
    // It no longer has protection from black sources; it still has it from red ones.
    t.g.recompute();
    assert!(!t.g.protected_from(paladin, black));
    assert!(t.g.protected_from(paladin, red));
}

#[test]
fn losing_bands_with_other_leaves_banding() {
    cr!("702.22b", "702.22a", "613.1f");
    ruling!(
        "Shelkin Brownie",
        "Can only remove “bands with other” and not normal “banding” ability."
    );
    let mut t = TestGame::new(2);
    // Ayesha Tanaka: a white legendary creature with banding; Cathedral of Serra gives it
    // "bands with other legendary creatures".
    let ayesha = t.battlefield(P1, "Ayesha Tanaka");
    t.battlefield(P1, "Cathedral of Serra");
    let banding = |t: &mut TestGame| instances(t, ayesha, KeywordKind::Banding);
    assert_eq!(banding(&mut t).len(), 2);
    let brownie = t.battlefield(P0, "Shelkin Brownie");
    t.activate(P0, brownie, 0, &[Entity::Object(ayesha)]).unwrap();
    t.resolve_all();
    let left = banding(&mut t);
    assert_eq!(left.len(), 1, "only the bands with other ability is lost");
    assert!(left[0].filter.is_none(), "plain banding stays");
    // It ends with the turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(banding(&mut t).len(), 2);
    // Losing banding, on the other hand, loses the "bands with other" ability as well.
    let breaker = crate::common_k702_011_017::custom_card(
        "Band Breaker",
        "Artifact",
        None,
        "{T}: Target creature loses banding until end of turn.",
    );
    let breaker = t.custom(P0, breaker, Zone::Battlefield);
    t.activate(P0, breaker, 0, &[Entity::Object(ayesha)]).unwrap();
    t.resolve_all();
    assert!(banding(&mut t).is_empty());
}

/// The candidates offered for the most recent target choice.
fn offered(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("a target choice")
}

/// The targets chosen for the spell or ability on top of the stack.
fn top_targets(t: &TestGame) -> Vec<Entity> {
    let top = *t.g.stack.last().expect("something on the stack");
    t.g.obj(top).stack.as_ref().unwrap().chosen[0].targets[0].clone()
}

#[test]
fn hexproof_from_activated_and_triggered_abilities() {
    cr!("702.11d", "702.11b", "602.2b", "603.3d");
    ruling!(
        "Volatile Stormdrake",
        "Volatile Stormdrake can't be the target of any activated or triggered abilities your opponents control."
    );
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Volatile Stormdrake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // An activated ability P0 controls: the Drake isn't a legal choice as it's activated
    // (another creature of the same player is).
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Object(drake)]).unwrap();
    let choices = offered(&t);
    assert!(choices.contains(&Entity::Object(bears)));
    assert!(!choices.contains(&Entity::Object(drake)));
    assert!(!top_targets(&t).contains(&Entity::Object(drake)));
    t.resolve_all();
    assert_eq!(t.obj_now(drake).damage, 0);
    // A triggered ability P0 controls (Flametongue Kavu's "deals 4 damage to target
    // creature"): the Drake isn't a legal choice as it's put on the stack.
    t.lands(P0, "Mountain", 4);
    let kavu = t.hand(P0, "Flametongue Kavu");
    t.cast(P0, kavu).go();
    t.answer_targets(P0, &[Entity::Object(drake)]);
    t.resolve(); // the Kavu; its enters trigger goes on the stack
    let choices = offered(&t);
    assert!(choices.contains(&Entity::Object(bears)));
    assert!(!choices.contains(&Entity::Object(drake)));
    assert!(!top_targets(&t).contains(&Entity::Object(drake)));
    t.resolve_all();
    assert!(t.on_battlefield(drake));
    // A spell can target it.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(drake).go();
    assert_eq!(top_targets(&t), vec![Entity::Object(drake)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Volatile Stormdrake"));
    // Its controller's own abilities can target it (CR 702.11b: opponents only).
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Volatile Stormdrake");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, sorcerer, 0, &[Entity::Object(drake)]).unwrap();
    assert_eq!(top_targets(&t), vec![Entity::Object(drake)]);
    t.resolve_all();
    assert_eq!(t.obj_now(drake).damage, 1);
}
