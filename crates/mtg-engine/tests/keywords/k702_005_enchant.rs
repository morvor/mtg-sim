//! CR 702.5 Enchant.

use super::k702_001_010_common::*;
use mtg_engine::ability::{Duration, Effect, Filter, Modification, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn attached_to(t: &TestGame, aura: ObjectId) -> Option<Entity> {
    t.obj_now(aura).attached_to
}

/// The single Aura named `name` on the battlefield.
fn aura_on_battlefield(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "{name} on the battlefield");
    v[0]
}

#[test]
fn enchant_parses_into_a_restriction_on_what_the_aura_can_enchant() {
    cr!("702.5a");
    let kws = printed_keywords("Pacifism", KeywordKind::Enchant);
    assert_eq!(kws.len(), 1);
    assert!(matches!(kws[0].filter, Some(Filter::Type(CardType::Creature))));
    for name in [
        "Controlled Instincts",
        "Curse of the Pierced Heart",
        "Psychic Possession",
        "Inferno Fist",
    ] {
        assert_eq!(printed_keywords(name, KeywordKind::Enchant).len(), 1, "{name}");
    }
    // Object phrases with commas stay one keyword.
    let kws = printed_keywords("Planar Disruption", KeywordKind::Enchant);
    assert_eq!(kws.len(), 1);
    assert!(matches!(&kws[0].filter, Some(Filter::Or(v)) if v.len() == 3));
    // "Enchant player" has no object quality.
    let kws = printed_keywords("Curse of the Pierced Heart", KeywordKind::Enchant);
    assert!(kws[0].filter.is_none());
}

#[test]
fn enchant_with_a_list_of_qualities_can_enchant_any_of_them() {
    cr!("702.5a");
    let mut t = TestGame::new(2);
    let rock = t.battlefield(P1, "Mind Stone");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 2);
    // Planar Disruption: "Enchant artifact, creature, or planeswalker".
    let disruption = t.hand(P0, "Planar Disruption");
    t.cast(P0, disruption).target(rock).go();
    let candidates = last_target_candidates(&t);
    assert!(candidates.contains(&Entity::Object(rock)));
    assert!(!candidates.contains(&Entity::Object(forest)));
    t.resolve();
    let aura = aura_on_battlefield(&t, "Planar Disruption");
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(rock)));
}

#[test]
fn aura_spell_can_target_only_what_its_enchant_ability_allows() {
    cr!("702.5a", "303.4a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 2);
    let pacifism = t.hand(P0, "Pacifism");
    // An artifact creature is a creature; a land isn't.
    t.cast(P0, pacifism).target(myr).go();
    let candidates = last_target_candidates(&t);
    assert!(candidates.contains(&Entity::Object(myr)));
    assert!(candidates.contains(&Entity::Object(bears)));
    assert!(!candidates.contains(&Entity::Object(forest)));
    assert!(!candidates.contains(&Entity::Player(P1)));
    t.resolve();
    let aura = aura_on_battlefield(&t, "Pacifism");
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(myr)));
    // It can't be moved onto a land.
    assert!(!t.g.attach(aura, Entity::Object(forest)));
}

#[test]
fn enchant_creature_you_control_cant_target_an_opponents_creature() {
    cr!("702.5a");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let fist = t.hand(P0, "Inferno Fist");
    t.cast(P0, fist).target(mine).go();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(mine)]);
    let _ = theirs;
    t.resolve();
    assert_eq!(t.pt(mine), (4, 2));
}

#[test]
fn aura_whose_object_stops_matching_its_enchant_ability_goes_to_the_graveyard() {
    cr!("702.5a", "704.5m");
    ruling!(
        "Controlled Instincts",
        "If at any time the enchanted creature is neither red nor green, Controlled Instincts is put into its owner’s graveyard as a state-based action."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Island", 2);
    let instincts = t.hand(P0, "Controlled Instincts");
    // Enchant red or green creature: not a blue creature.
    t.cast(P0, instincts).target(bears).go();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(bears)]);
    let _ = drake;
    t.resolve();
    let aura = aura_on_battlefield(&t, "Controlled Instincts");
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
    // The Bears become blue.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::single(Color::Blue))],
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    t.settle();
    assert!(t.in_graveyard(P0, "Controlled Instincts"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn auras_follow_the_rules_for_auras() {
    cr!("702.5b", "303.4b", "303.4c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let pacifism = t.hand(P0, "Pacifism");
    t.cast(P0, pacifism).target(bears).go();
    t.resolve();
    let aura = aura_on_battlefield(&t, "Pacifism");
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
    assert!(!t.g.can_block_at_all(bears));
    // The enchanted object leaves: the Aura is put into its owner's graveyard.
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Pacifism"));
}

fn doubly_enchanting_aura() -> CardDef {
    custom_card(
        "Twofold Binding",
        "Enchantment — Aura",
        "{W}",
        None,
        "Enchant creature\nEnchant artifact\nEnchanted permanent can't attack or block.",
    )
}

#[test]
fn every_instance_of_enchant_applies() {
    cr!("702.5c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rock = t.battlefield(P1, "Mind Stone");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.lands(P0, "Plains", 3);
    let aura = t.custom(P0, doubly_enchanting_aura(), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(instances(&t, aura, KeywordKind::Enchant), 2);
    // A creature that isn't an artifact, and an artifact that isn't a creature, are both
    // illegal targets; an artifact creature is fine.
    t.cast(P0, aura).target(myr).go();
    assert_eq!(last_target_candidates(&t), vec![Entity::Object(myr)]);
    t.resolve();
    let on = aura_on_battlefield(&t, "Twofold Binding");
    assert_eq!(attached_to(&t, on), Some(Entity::Object(myr)));
    // It can't be moved onto something that matches only one of them.
    assert!(!t.g.attach(on, Entity::Object(bears)));
    assert!(!t.g.attach(on, Entity::Object(rock)));
    // If the Myr stops being an artifact, the Aura is put into the graveyard.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveTypes(vec![CardType::Artifact])],
            duration: Duration::EndOfTurn,
        },
        &[myr],
    );
    t.settle();
    assert!(t.in_graveyard(P0, "Twofold Binding"));
}

#[test]
fn aura_that_enchants_a_player_targets_and_attaches_to_players_only() {
    cr!("702.5d", "303.4a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let curse = t.hand(P0, "Curse of the Pierced Heart");
    t.cast(P0, curse).target(P1).go();
    let candidates = last_target_candidates(&t);
    assert!(candidates.contains(&Entity::Player(P0)));
    assert!(candidates.contains(&Entity::Player(P1)));
    assert!(!candidates.contains(&Entity::Object(bears)));
    t.resolve();
    let aura = aura_on_battlefield(&t, "Curse of the Pierced Heart");
    assert_eq!(attached_to(&t, aura), Some(Entity::Player(P1)));
    // It can't become attached to a permanent.
    assert!(!t.g.attach(aura, Entity::Object(bears)));
    assert_eq!(attached_to(&t, aura), Some(Entity::Player(P1)));
    assert!(t.g.attach(aura, Entity::Player(P0)));
    assert_eq!(attached_to(&t, aura), Some(Entity::Player(P0)));
}

#[test]
fn enchant_opponent_can_enchant_only_an_opponent() {
    cr!("702.5d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let possession = t.hand(P0, "Psychic Possession");
    t.cast(P0, possession).target(P1).go();
    assert_eq!(last_target_candidates(&t), vec![Entity::Player(P1)]);
    t.resolve();
    let aura = aura_on_battlefield(&t, "Psychic Possession");
    assert_eq!(attached_to(&t, aura), Some(Entity::Player(P1)));
    assert!(!t.g.attach(aura, Entity::Player(P0)));
}
