//! One-shot "becomes" effects (CR 611.2a, 205.1): planeswalkers that become creatures
//! ("... that's still a planeswalker"), animated lands ("It's still a land."), creature
//! type and basic land type changes, for a stated duration or indefinitely.

use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for name in names {
        let c = card(name);
        assert!(
            c.unsupported_text().is_empty(),
            "{name} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// The index (among its activated abilities, for `TestGame::activate`) of the ability of
/// `id` whose text starts with `prefix`.
fn ability(t: &TestGame, id: ObjectId, prefix: &str) -> usize {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with(prefix))
        .unwrap_or_else(|| panic!("no ability starting with {prefix:?}"))
}

fn is(t: &TestGame, id: ObjectId, ty: CardType) -> bool {
    t.obj_now(id).chars.card_types.contains(ty)
}

fn has_subtype(t: &TestGame, id: ObjectId, st: &str) -> bool {
    t.obj_now(id).chars.has_subtype(st)
}

// ---------------------------------------------------------------------------
// Planeswalkers that become creatures
// ---------------------------------------------------------------------------

#[test]
fn gideon_becomes_a_creature_thats_still_a_planeswalker_until_end_of_turn() {
    cr!("205.1b", "611.2a", "613.1d", "613.4b");
    ruling!(
        "Gideon Jura",
        "He remains a planeswalker with the planeswalker type Gideon."
    );
    assert_supported(&[
        "Gideon, Ally of Zendikar",
        "Gideon, Martial Paragon",
    ]);
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Jura");
    assert!(!is(&t, gideon, CardType::Creature));
    let i = ability(&t, gideon, "0:");
    t.activate(P0, gideon, i, &[]).unwrap();
    t.resolve();
    assert!(is(&t, gideon, CardType::Creature));
    assert!(is(&t, gideon, CardType::Planeswalker));
    assert!(has_subtype(&t, gideon, "Human"));
    assert!(has_subtype(&t, gideon, "Soldier"));
    assert!(has_subtype(&t, gideon, "Gideon"));
    assert_eq!(t.pt(gideon), (6, 6));
    assert_eq!(t.counters(gideon, "loyalty"), 6);
    // "Prevent all damage that would be dealt to him this turn."
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(gideon).go();
    t.resolve();
    assert!(t.on_battlefield(gideon));
    assert_eq!(t.counters(gideon, "loyalty"), 6);
    assert_eq!(t.obj_now(gideon).damage, 0);
    // He can attack as a creature.
    t.attack(&[(gideon, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 14);
    // The effect ends at end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!is(&t, gideon, CardType::Creature));
    assert!(is(&t, gideon, CardType::Planeswalker));
    assert!(!has_subtype(&t, gideon, "Human"));
}

#[test]
fn a_planeswalker_becoming_a_creature_doesnt_enter_the_battlefield() {
    cr!("611.2a");
    ruling!(
        "Gideon Jura",
        "that doesn't count as having a creature enter"
    );
    let mut t = TestGame::new(2);
    // "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let gideon = t.battlefield(P0, "Gideon Jura");
    let i = ability(&t, gideon, "0:");
    t.activate(P0, gideon, i, &[]).unwrap();
    t.resolve_all();
    assert!(is(&t, gideon, CardType::Creature));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn sarkhan_becomes_a_dragon_and_is_no_longer_a_planeswalker() {
    cr!("205.1a", "120.3c", "613.1d");
    ruling!(
        "Sarkhan, the Dragonspeaker",
        "Because he's not a planeswalker at this time, damage dealt to him won't cause those counters to be removed."
    );
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, the Dragonspeaker");
    let i = ability(&t, sarkhan, "+1:");
    t.activate(P0, sarkhan, i, &[]).unwrap();
    assert_eq!(t.counters(sarkhan, "loyalty"), 5);
    t.resolve();
    // "a legendary 4/4 red Dragon creature with flying, indestructible, and haste": the
    // new card type replaces the old ones.
    assert!(is(&t, sarkhan, CardType::Creature));
    assert!(!is(&t, sarkhan, CardType::Planeswalker));
    assert!(has_subtype(&t, sarkhan, "Dragon"));
    assert!(!has_subtype(&t, sarkhan, "Sarkhan"));
    assert_eq!(t.pt(sarkhan), (4, 4));
    let chars = &t.obj_now(sarkhan).chars;
    assert!(chars.colors.contains(Color::Red) && chars.colors.count() == 1);
    // Damage doesn't remove loyalty counters from a non-planeswalker (and he's
    // indestructible).
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(sarkhan).go();
    t.resolve();
    assert!(t.on_battlefield(sarkhan));
    assert_eq!(t.counters(sarkhan, "loyalty"), 5);
    assert_eq!(t.obj_now(sarkhan).damage, 3);
    // At end of turn he's a planeswalker again, with his loyalty.
    t.advance_to(P1, Step::Upkeep);
    assert!(is(&t, sarkhan, CardType::Planeswalker));
    assert!(!is(&t, sarkhan, CardType::Creature));
    assert_eq!(t.counters(sarkhan, "loyalty"), 5);
}

// ---------------------------------------------------------------------------
// Animated lands ("It's still a land.")
// ---------------------------------------------------------------------------

#[test]
fn koth_animates_a_mountain_that_is_still_a_land() {
    cr!("205.1b", "105.3", "611.2a");
    ruling!(
        "Koth of the Hammer",
        "Koth’s first ability can target any Mountain, including an untapped Mountain"
    );
    let mut t = TestGame::new(2);
    let koth = t.battlefield(P0, "Koth of the Hammer");
    let mountain = t.battlefield(P0, "Mountain");
    t.g.objects[mountain.0 as usize].tapped = true;
    let i = ability(&t, koth, "+1:");
    t.activate(P0, koth, i, &[Entity::Object(mountain)]).unwrap();
    t.resolve();
    // "Untap target Mountain. It becomes a 4/4 red Elemental creature until end of turn.
    // It's still a land."
    assert!(!t.obj_now(mountain).tapped);
    assert!(is(&t, mountain, CardType::Creature));
    assert!(is(&t, mountain, CardType::Land));
    assert!(has_subtype(&t, mountain, "Mountain"));
    assert!(has_subtype(&t, mountain, "Elemental"));
    assert_eq!(t.pt(mountain), (4, 4));
    assert!(t.obj_now(mountain).chars.colors.contains(Color::Red));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is(&t, mountain, CardType::Creature));
    assert!(is(&t, mountain, CardType::Land));
}

#[test]
fn nissa_animates_a_land_until_her_controllers_next_turn() {
    cr!("205.1b", "611.2a");
    ruling!(
        "Nissa, Vital Force",
        "the target land still has any abilities it had before it became a creature and any other types it had"
    );
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Vital Force");
    let forest = t.battlefield(P0, "Forest");
    let i = ability(&t, nissa, "+1:");
    t.activate(P0, nissa, i, &[Entity::Object(forest)]).unwrap();
    t.resolve();
    assert!(is(&t, forest, CardType::Creature));
    assert!(has_subtype(&t, forest, "Forest"));
    assert_eq!(t.pt(forest), (5, 5));
    // Still a creature during the opponent's turn...
    t.advance_to(P1, Step::Upkeep);
    assert!(is(&t, forest, CardType::Creature));
    // ...but not once its controller's next turn begins.
    t.advance_to(P0, Step::Upkeep);
    assert!(!is(&t, forest, CardType::Creature));
    assert!(is(&t, forest, CardType::Land));
}

#[test]
fn mishras_factory_becomes_an_assembly_worker_and_keeps_its_abilities() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Mishra's Factory",
        "After it becomes a creature, Mishra's Factory will still have all its abilities."
    );
    assert_supported(&[
        "Mishra's Factory",
        "Treetop Village",
        "Celestial Colonnade",
        "Inkmoth Nexus",
        "Restless Cottage",
    ]);
    let mut t = TestGame::new(2);
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.lands(P0, "Wastes", 1);
    let i = ability(&t, factory, "{1}:");
    t.activate(P0, factory, i, &[]).unwrap();
    t.resolve();
    assert!(is(&t, factory, CardType::Artifact));
    assert!(is(&t, factory, CardType::Creature));
    assert!(is(&t, factory, CardType::Land));
    assert!(has_subtype(&t, factory, "Assembly-Worker"));
    assert_eq!(t.pt(factory), (2, 2));
    // "{T}: Target Assembly-Worker creature gets +1/+1 until end of turn." on itself (the
    // automatic payment of {1} may have tapped the Factory itself for {C}).
    t.g.objects[factory.0 as usize].tapped = false;
    let pump = ability(&t, factory, "{T}: Target");
    t.activate(P0, factory, pump, &[Entity::Object(factory)])
        .unwrap();
    t.resolve();
    assert_eq!(t.pt(factory), (3, 3));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is(&t, factory, CardType::Creature));
    assert!(!is(&t, factory, CardType::Artifact));
}

#[test]
fn mutavault_has_all_creature_types_but_not_changeling() {
    cr!("205.3m", "613.1d");
    ruling!(
        "Mutavault",
        "it does not have the changeling keyword ability"
    );
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    let i = ability(&t, vault, "{1}:");
    t.activate(P0, vault, i, &[]).unwrap();
    t.resolve();
    assert!(is(&t, vault, CardType::Creature));
    assert!(is(&t, vault, CardType::Land));
    assert!(has_subtype(&t, vault, "Goblin"));
    assert!(has_subtype(&t, vault, "Sliver"));
    assert_eq!(t.pt(vault), (2, 2));
    assert!(!t
        .obj_now(vault)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Changeling));
}

// ---------------------------------------------------------------------------
// Type changes without "still"
// ---------------------------------------------------------------------------

#[test]
fn opal_champion_becomes_a_creature_and_is_no_longer_an_enchantment() {
    cr!("205.1a", "603.4");
    ruling!(
        "Opal Champion",
        "When it turns into a creature, it is no longer an enchantment."
    );
    assert_supported(&["Opal Champion", "Opal Acrolith", "Veil of Birds"]);
    let mut t = TestGame::new(2);
    let opal = t.battlefield(P0, "Opal Champion");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    t.settle();
    // The trigger resolves before the creature spell.
    t.resolve();
    assert!(t.g.stack.contains(&spell));
    assert!(is(&t, opal, CardType::Creature));
    assert!(!is(&t, opal, CardType::Enchantment));
    assert!(has_subtype(&t, opal, "Knight"));
    assert_eq!(t.pt(opal), (3, 3));
    // The effect has no duration: it lasts (CR 611.2a).
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    assert!(is(&t, opal, CardType::Creature));
    // The spell itself didn't change.
    let bear = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.pt(bear), (2, 2));
    assert!(!has_subtype(&t, bear, "Knight"));
}

#[test]
fn becoming_a_creature_type_replaces_the_old_creature_types() {
    cr!("205.1a", "611.2a");
    ruling!(
        "Boldwyr Intimidator",
        "Each of the activated abilities replaces all creature types the affected creature may have had."
    );
    assert_supported(&["Boldwyr Intimidator", "Kargan Intimidator", "Serpentine Ambush"]);
    let mut t = TestGame::new(2);
    let boldwyr = t.battlefield(P0, "Boldwyr Intimidator");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let i = ability(&t, boldwyr, "{R}:");
    t.activate(P0, boldwyr, i, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(has_subtype(&t, bears, "Coward"));
    assert!(!has_subtype(&t, bears, "Bear"));
    assert_eq!(t.pt(bears), (2, 2));
    t.advance_to(P1, Step::Upkeep);
    assert!(has_subtype(&t, bears, "Bear"));
    assert!(!has_subtype(&t, bears, "Coward"));
}

#[test]
fn a_land_that_becomes_an_island_loses_its_other_land_types() {
    cr!("305.7", "611.2a");
    assert_supported(&["Streambed Aquitects", "Tidal Warrior", "Kavu Recluse"]);
    let mut t = TestGame::new(2);
    let aquitects = t.battlefield(P0, "Streambed Aquitects");
    let forest = t.battlefield(P1, "Forest");
    let i = ability(&t, aquitects, "{T}: Target land");
    t.activate(P0, aquitects, i, &[Entity::Object(forest)])
        .unwrap();
    t.resolve();
    assert!(has_subtype(&t, forest, "Island"));
    assert!(!has_subtype(&t, forest, "Forest"));
    assert!(is(&t, forest, CardType::Land));
    t.advance_to(P1, Step::Upkeep);
    assert!(has_subtype(&t, forest, "Forest"));
    assert!(!has_subtype(&t, forest, "Island"));
}
