//! CR 702.18 Shroud.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn shroud_permanent_cant_be_targeted_by_anyone() {
    cr!("702.18", "702.18a");
    assert_supported("Kalonian Behemoth");
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Kalonian Behemoth");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    let own = t.battlefield(P0, "Prodigal Pyromancer");
    // Unlike hexproof, shroud stops its controller's spells and abilities too.
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", behemoth));
    assert!(!spell_can_target(&mut t, P0, "Giant Growth", behemoth));
    assert!(!ability_can_target(&mut t, pyromancer, 0, behemoth));
    assert!(!ability_can_target(&mut t, own, 0, behemoth));
    // An Aura spell targets what it will enchant.
    assert!(!spell_can_target(&mut t, P0, "Rancor", behemoth));
    // The permanent is still affected by spells and abilities that don't target it.
    let enchantress = t.battlefield(P0, "Argothian Enchantress");
    assert!(t.obj_now(enchantress).has_keyword(KeywordKind::Shroud));
    t.lands(P1, "Mountain", 2);
    let pyroclasm = t.hand(P1, "Pyroclasm");
    t.set_step(P1, turn::Step::PrecombatMain);
    t.cast(P1, pyroclasm).go();
    t.resolve_all();
    assert!(!t.on_battlefield(enchantress));
}

#[test]
fn shroud_player_cant_be_targeted_by_anyone() {
    cr!("702.18a");
    assert_supported("True Believer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "True Believer");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", P0));
    assert!(!ability_can_target(&mut t, pyromancer, 0, P0));
    // Not even its controller can target themselves.
    assert!(!spell_can_target(&mut t, P0, "Lightning Bolt", P0));
    assert!(spell_can_target(&mut t, P0, "Lightning Bolt", P1));
    // The player's permanents can still be targeted.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", bears));
}

#[test]
fn gaining_shroud_in_response_makes_the_target_illegal() {
    cr!("702.18a");
    ruling!(
        "Glimmering Angel",
        "You can activate the ability that grants shroud in response to this card being targeted"
    );
    assert_supported("Glimmering Angel");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Glimmering Angel");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(angel).go();
    t.lands(P0, "Island", 1);
    t.activate(P0, angel, 0, &[]).unwrap();
    t.resolve();
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Shroud));
    t.resolve_all();
    // The Bolt's only target is illegal: it doesn't resolve.
    assert!(t.on_battlefield(angel));
    assert_eq!(t.obj_now(angel).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn shroud_only_works_on_the_battlefield() {
    cr!("702.18a");
    ruling!(
        "Blurred Mongoose",
        "It can be targeted while on the stack. Shroud only works while it is on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let mongoose = t.hand(P0, "Blurred Mongoose");
    let spell = t.cast(P0, mongoose).go();
    assert!(spell_can_target(&mut t, P1, "Essence Scatter", spell));
    t.resolve_all();
    let perm = t.g.current(spell);
    assert!(t.on_battlefield(perm));
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", perm));
    // A creature card with shroud in a graveyard can be targeted.
    let gy = t.graveyard(P0, "Blurred Mongoose");
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
    assert!(spell_can_target(&mut t, P0, "Raise Dead", gy));
}

#[test]
fn shroud_doesnt_remove_auras_already_attached() {
    cr!("702.18a");
    ruling!(
        "Robe of Mirrors",
        "Having shroud will prevent the creature from being targeted by Aura spells, but it does not remove any Auras that are already attached"
    );
    assert_supported("Robe of Mirrors");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An opponent's Pacifism is already on it.
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, Entity::Object(bears));
    assert_eq!(t.obj_now(pacifism).attached_to, Some(Entity::Object(bears)));
    // The bears get shroud from Robe of Mirrors.
    t.lands(P0, "Island", 1);
    let robe = t.hand(P0, "Robe of Mirrors");
    t.cast(P0, robe).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Shroud));
    // Pacifism stays attached; new Aura spells can't target it.
    assert!(t.on_battlefield(pacifism));
    assert_eq!(t.obj_now(pacifism).attached_to, Some(Entity::Object(bears)));
    assert!(!t.g.can_attack(bears));
    assert!(!spell_can_target(&mut t, P0, "Rancor", bears));
}

#[test]
fn multiple_instances_of_shroud_are_redundant() {
    cr!("702.18b");
    let mut t = TestGame::new(2);
    let enchantress = t.battlefield(P0, "Argothian Enchantress");
    let greaves = t.battlefield(P0, "Lightning Greaves");
    t.g.attach(greaves, Entity::Object(enchantress));
    t.g.recompute();
    assert_eq!(keyword_count(&t, enchantress, KeywordKind::Shroud), 2);
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", enchantress));
    // Losing one instance leaves it with shroud: the other alone does the same.
    t.g.unattach(greaves);
    t.g.recompute();
    assert_eq!(keyword_count(&t, enchantress, KeywordKind::Shroud), 1);
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", enchantress));
    // Two sources of player shroud work like one.
    t.battlefield(P1, "True Believer");
    t.battlefield(P1, "Ivory Mask");
    assert!(!spell_can_target(&mut t, P0, "Lightning Bolt", P1));
}
