//! CR 702.6 Equip.

use super::k702_001_010_common::*;
use mtg_engine::ability::{Duration, Effect, Modification, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn attached_to(t: &TestGame, eq: ObjectId) -> Option<Entity> {
    t.obj_now(eq).attached_to
}

#[test]
fn equip_attaches_to_target_creature_you_control() {
    cr!("702.6a");
    let kws = printed_keywords("Short Sword", KeywordKind::Equip);
    assert_eq!(kws.len(), 1);
    assert!(kws[0].cost.is_some());
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    t.activate(P0, sword, 0, &[Entity::Object(bears)]).unwrap();
    // Only creatures its controller controls, and not the Equipment itself.
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(bears)]);
    let _ = theirs;
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 3));
    // Equipping another creature moves it.
    let giant = t.battlefield(P0, "Hill Giant");
    t.activate(P0, sword, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(giant)));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn equip_can_be_activated_only_as_a_sorcery() {
    cr!("702.6a");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    // During the opponent's turn.
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
    t.clear_answers();
    // During combat on its controller's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
    t.clear_answers();
    // With something on the stack in a main phase.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
    t.clear_answers();
    t.resolve_all();
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_ok());
}

#[test]
fn equipment_follows_the_rules_for_equipment() {
    cr!("702.6b", "301.5c", "301.5d", "704.5n");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, sword, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    // The equipped creature changing control doesn't change control of the Equipment.
    apply(
        &mut t,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: mtg_engine::ability::PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[bears],
    );
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(sword).controller, P0);
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(bears)));
    // The creature leaves: the Equipment stays on the battlefield, unattached.
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.on_battlefield(sword));
    assert_eq!(attached_to(&t, sword), None);
}

#[test]
fn equip_quality_restricts_targets_but_not_what_it_stays_attached_to() {
    cr!("702.6c");
    ruling!(
        "Blackblade Reforged",
        "Whether the target creature is legendary is checked only as Blackblade Reforged's first equip ability is activated and as that ability resolves. If the creature somehow becomes nonlegendary later, Blackblade Reforged remains attached to it."
    );
    ruling!(
        "Blackblade Reforged",
        "\"Equip [quality] creature\" is a variant of the equip keyword."
    );
    let kws = printed_keywords("Blackblade Reforged", KeywordKind::Equip);
    assert_eq!(kws.len(), 2);
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Blackblade Reforged");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    // "Equip legendary creature {3}": only the legendary creature.
    t.activate(P0, blade, 0, &[Entity::Object(isamaru)]).unwrap();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(isamaru)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(isamaru)));
    // It stops being legendary: the Equipment stays attached.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveSupertypes(vec![Supertype::Legendary])],
            duration: Duration::EndOfTurn,
        },
        &[isamaru],
    );
    t.settle();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(isamaru)));
    let _ = bears;
}

#[test]
fn equip_creature_type_quality() {
    cr!("702.6c");
    assert_supported("Robe of the Archmagi");
    let mut t = TestGame::new(2);
    // Robe of the Archmagi: "Equip {4}" and "Equip Shaman, Warlock, or Wizard {1}".
    let robe = t.battlefield(P0, "Robe of the Archmagi");
    let wizard = t.battlefield(P0, "Prodigal Pyromancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.activate(P0, robe, 1, &[Entity::Object(wizard)]).unwrap();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(wizard)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, robe), Some(Entity::Object(wizard)));
    let _ = bears;
}

#[test]
fn equip_commander_can_target_only_a_commander() {
    cr!("702.6c");
    assert_supported("Unstable Molecule Suit");
    let mut t = TestGame::new(2);
    let suit = t.battlefield(P0, "Unstable Molecule Suit");
    let commander = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.objects[commander.0 as usize].is_commander = true;
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    // "Equip commander {2}" is its first ability.
    t.activate(P0, suit, 0, &[Entity::Object(commander)]).unwrap();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(commander)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, suit), Some(Entity::Object(commander)));
    let _ = bears;
}

#[test]
fn any_of_several_equip_abilities_may_be_activated() {
    cr!("702.6d");
    let mut t = TestGame::new(2);
    let robe = t.battlefield(P0, "Robe of the Archmagi");
    let wizard = t.battlefield(P0, "Prodigal Pyromancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    // "Equip {4}": any creature you control.
    t.activate(P0, robe, 0, &[Entity::Object(bears)]).unwrap();
    let candidates = last_target_candidates(&t);
    assert!(candidates.contains(&Entity::Object(bears)));
    assert!(candidates.contains(&Entity::Object(wizard)));
    t.resolve_all();
    assert_eq!(attached_to(&t, robe), Some(Entity::Object(bears)));
    // "Equip Shaman, Warlock, or Wizard {1}" moves it to the Wizard for {1}.
    t.activate(P0, robe, 1, &[Entity::Object(wizard)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, robe), Some(Entity::Object(wizard)));
    let untapped = t
        .g
        .battlefield
        .iter()
        .filter(|id| t.obj(**id).chars.is_land() && !t.obj(**id).tapped)
        .count();
    assert_eq!(untapped, 0);
}

#[test]
fn equip_planeswalker_attaches_as_though_it_were_a_creature() {
    cr!("702.6e", "208.5");
    ruling!(
        "Luxior, Giada's Gift",
        "“Equip planeswalker” is a variant of the equip ability."
    );
    ruling!(
        "Luxior, Giada's Gift",
        "Unless another effect is making that planeswalker a creature with a power and toughness, the equipped planeswalker has base power and toughness 0/0."
    );
    assert_supported("Luxior, Giada's Gift");
    let mut t = TestGame::new(2);
    let luxior = t.battlefield(P0, "Luxior, Giada's Gift");
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Ajani Goldmane");
    assert_eq!(t.counters(jace, "loyalty"), 3);
    t.lands(P0, "Plains", 1);
    // "Equip planeswalker {1}" (its first equip ability) targets a planeswalker you
    // control, not a creature.
    t.activate(P0, luxior, 0, &[Entity::Object(jace)]).unwrap();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(jace)]);
    let _ = (bears, theirs);
    t.resolve_all();
    assert_eq!(attached_to(&t, luxior), Some(Entity::Object(jace)));
    // Luxior makes it a 0/0 creature that gets +1/+1 for each counter on it.
    let j = t.obj_now(jace);
    assert!(j.is_creature());
    assert!(!j.is(CardType::Planeswalker));
    assert_eq!(t.pt(jace), (3, 3));
    assert!(t.on_battlefield(jace));
}

#[test]
fn a_planeswalker_equipped_by_ordinary_equipment_rules_becomes_unattached() {
    cr!("702.6e", "704.5n");
    let mut t = TestGame::new(2);
    let luxior = t.battlefield(P0, "Luxior, Giada's Gift");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.lands(P0, "Plains", 1);
    t.activate(P0, luxior, 0, &[Entity::Object(jace)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, luxior), Some(Entity::Object(jace)));
    // Without Luxior's type-changing ability, the planeswalker isn't a creature, so the
    // Equipment is attached to an illegal permanent.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[luxior],
    );
    t.settle();
    assert_eq!(attached_to(&t, luxior), None);
    assert!(t.on_battlefield(luxior));
    assert!(t.obj_now(jace).is(CardType::Planeswalker));
}

#[test]
fn equip_does_nothing_if_the_target_is_no_longer_controlled_by_you() {
    cr!("702.6a", "301.5b", "608.2b");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.activate(P0, sword, 0, &[Entity::Object(bears)]).unwrap();
    // In response, the opponent gains control of the Bears.
    apply(
        &mut t,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: mtg_engine::ability::PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), None);
}

#[test]
fn equipping_the_creature_its_already_attached_to_does_nothing() {
    cr!("702.6a", "701.3b");
    ruling!(
        "Killer Cosplay",
        "An Equipment can't become attached to a creature if it's already attached to it. That is, you can activate the equip ability targeting the creature Killer Cosplay's already attached to, but that ability resolving won't do anything."
    );
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, sword, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let ts = t.obj_now(sword).timestamp;
    t.activate(P0, sword, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(bears)));
    assert_eq!(t.obj_now(sword).timestamp, ts, "it didn't become attached again");
    assert_eq!(t.pt(bears), (3, 3));
}
