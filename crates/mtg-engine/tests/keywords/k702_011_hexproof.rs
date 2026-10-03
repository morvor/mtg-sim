//! CR 702.11 Hexproof.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hexproof_permanent_cant_be_targeted_by_opponents_spells_or_abilities() {
    cr!("702.11", "702.11b");
    assert_supported("Slippery Bogle");
    let mut t = TestGame::new(2);
    let bogle = t.battlefield(P0, "Slippery Bogle");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    // An opponent's spells and abilities can't target it...
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", bogle));
    assert!(!ability_can_target(&mut t, pyromancer, 0, bogle));
    // ...but its controller's can.
    assert!(spell_can_target(&mut t, P0, "Giant Growth", bogle));
    assert!(spell_can_target(&mut t, P0, "Lightning Bolt", bogle));
    let own = t.battlefield(P0, "Prodigal Pyromancer");
    assert!(ability_can_target(&mut t, own, 0, bogle));
    // Effects that don't target still affect it.
    t.lands(P1, "Mountain", 2);
    let pyroclasm = t.hand(P1, "Pyroclasm");
    t.set_step(P1, turn::Step::PrecombatMain);
    t.cast(P1, pyroclasm).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bogle));
}

#[test]
fn hexproof_is_a_static_ability_that_applies_while_granted() {
    cr!("702.11a", "702.11b");
    assert_supported("Sheltering Word");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    // In response, the bears gain hexproof: the Bolt's only target is now illegal.
    t.lands(P0, "Forest", 2);
    let word = t.hand(P0, "Sheltering Word");
    t.cast(P0, word).target(bears).go();
    t.resolve();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // It lasts only until end of turn.
    t.advance_to(P1, turn::Step::Upkeep);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", bears));
}

#[test]
fn hexproof_player_cant_be_targeted_by_opponents() {
    cr!("702.11c");
    assert_supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Sanctity");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", P0));
    assert!(!ability_can_target(&mut t, pyromancer, 0, P0));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", P1));
    // The player can still target themselves.
    assert!(spell_can_target(&mut t, P0, "Lightning Bolt", P0));
    // Hexproof on a player doesn't protect their permanents.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", bears));
}

#[test]
fn hexproof_from_a_quality_stops_only_opponents_spells_and_sources_with_it() {
    cr!("702.11d");
    ruling!(
        "Knight of Grace",
        "can't be the target of black spells your opponents control or abilities of black sources your opponents control"
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Grace");
    assert_eq!(
        t.obj_now(knight)
            .chars
            .keyword(KeywordKind::Hexproof)
            .and_then(|k| k.filter.clone())
            .map(|f| format!("{f:?}")),
        Some("Color(Black)".to_string())
    );
    // Black spells and abilities of black sources an opponent controls can't target it.
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
    let assassin = t.battlefield(P1, "Royal Assassin");
    t.g.tap(knight);
    assert!(!ability_can_target(&mut t, assassin, 0, knight));
    // Nonblack ones can.
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", knight));
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(ability_can_target(&mut t, pyromancer, 0, knight));
    // Its controller's black spells and abilities can.
    assert!(spell_can_target(&mut t, P0, "Disfigure", knight));
    let own_assassin = t.battlefield(P0, "Royal Assassin");
    assert!(ability_can_target(&mut t, own_assassin, 0, knight));
}

#[test]
fn losing_hexproof_loses_hexproof_from_too() {
    cr!("702.11e");
    ruling!(
        "Knight of Grace",
        "If an effect says that a creature loses hexproof or can be targeted as though it didn't have hexproof, this applies to hexproof from black as well."
    );
    assert_supported("Shadowspear");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Grace");
    let bogle = t.battlefield(P0, "Slippery Bogle");
    let spear = t.battlefield(P1, "Shadowspear");
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
    // "{1}: Permanents your opponents control lose hexproof and indestructible until end
    // of turn."
    t.lands(P1, "Wastes", 1);
    t.activate(P1, spear, 0, &[]).unwrap();
    t.resolve();
    assert!(!t.obj_now(knight).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(bogle).has_keyword(KeywordKind::Hexproof));
    assert!(spell_can_target(&mut t, P1, "Disfigure", knight));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", bogle));
}

#[test]
fn losing_hexproof_is_locked_to_the_permanents_there_at_resolution() {
    cr!("702.11e", "611.2c");
    ruling!(
        "Shadowspear",
        "If a permanent enters the battlefield under an opponent's control with hexproof or indestructible after Shadowspear's second ability resolves, it won't lose that ability"
    );
    let mut t = TestGame::new(2);
    let spear = t.battlefield(P1, "Shadowspear");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, spear, 0, &[]).unwrap();
    t.resolve();
    let knight = t.battlefield(P0, "Knight of Grace");
    assert!(t.obj_now(knight).has_keyword(KeywordKind::Hexproof));
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
}

#[test]
fn targeting_as_though_no_hexproof_ignores_hexproof_from() {
    cr!("702.11e");
    ruling!(
        "Knight of Grace",
        "can be targeted as though it didn't have hexproof, this applies to hexproof from black as well"
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Grace");
    let bogle = t.battlefield(P0, "Slippery Bogle");
    // Glaring Spotlight: "Creatures your opponents control with hexproof can be the
    // targets of spells and abilities you control as though they didn't have hexproof."
    t.battlefield(P1, "Glaring Spotlight");
    assert!(spell_can_target(&mut t, P1, "Disfigure", knight));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", bogle));
    let assassin = t.battlefield(P1, "Royal Assassin");
    t.g.tap(knight);
    assert!(ability_can_target(&mut t, assassin, 0, knight));
    // Only for the Spotlight's controller: a third party's spells still can't.
    let mut t = TestGame::new(3);
    let knight = t.battlefield(P0, "Knight of Grace");
    t.battlefield(P1, "Glaring Spotlight");
    assert!(!spell_can_target(&mut t, P2, "Disfigure", knight));
    assert!(spell_can_target(&mut t, P1, "Disfigure", knight));
}

#[test]
fn targeting_players_and_permanents_as_though_no_hexproof() {
    cr!("702.11b", "702.11c", "702.11e");
    ruling!(
        "Kaya, Bane of the Dead",
        "and Kaya leaves the battlefield while that spell or ability is on the stack, that player or permanent becomes an illegal target"
    );
    assert_supported("Kaya, Bane of the Dead");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Leyline of Sanctity");
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let knight = t.battlefield(P1, "Knight of Grace");
    // "Your opponents and permanents your opponents control with hexproof can be the
    // targets of spells and abilities you control as though they didn't have hexproof."
    let kaya = t.battlefield(P0, "Kaya, Bane of the Dead");
    assert!(spell_can_target(&mut t, P0, "Lightning Bolt", P1));
    assert!(spell_can_target(&mut t, P0, "Lightning Bolt", bogle));
    assert!(spell_can_target(&mut t, P0, "Disfigure", knight));
    // Other opponents of the hexproof player still can't.
    assert!(!spell_can_target(&mut t, P2, "Lightning Bolt", P1));
    assert!(!spell_can_target(&mut t, P2, "Lightning Bolt", bogle));
    // If Kaya leaves while the spell is on the stack, the target becomes illegal.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Swamp", 3);
    let downfall = t.hand(P1, "Hero's Downfall");
    t.cast(P1, downfall).target(kaya).go();
    t.resolve_all();
    assert!(!t.on_battlefield(kaya));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn looking_for_hexproof_finds_hexproof_from() {
    cr!("702.11e");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Grace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let f = mtg_engine::ability::Filter::HasKeyword(KeywordKind::Hexproof);
    let ctx = mtg_engine::eval::Ctx::new(None, P1);
    assert!(t.g.matches(knight, &f, &ctx));
    assert!(!t.g.matches(bears, &f, &ctx));
}

#[test]
fn hexproof_from_two_qualities_is_two_abilities() {
    cr!("702.11f");
    let mut t = TestGame::new(2);
    // Raddic, Tal Zealot: "Hexproof from white and from black".
    let raddic = t.battlefield(P0, "Raddic, Tal Zealot");
    assert_eq!(keyword_count(&t, raddic, KeywordKind::Hexproof), 2);
    assert!(!spell_can_target(&mut t, P1, "Swords to Plowshares", raddic));
    assert!(!spell_can_target(&mut t, P1, "Disfigure", raddic));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", raddic));
    // Both are hexproof abilities: losing hexproof loses them both (CR 702.11e).
    let spear = t.battlefield(P1, "Shadowspear");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, spear, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(keyword_count(&t, raddic, KeywordKind::Hexproof), 0);
    assert!(spell_can_target(&mut t, P1, "Disfigure", raddic));
}

#[test]
fn hexproof_from_each_color_is_one_ability_per_color() {
    cr!("702.11g");
    let mut t = TestGame::new(2);
    // Breaker of Creation: "Hexproof from each color".
    let breaker = t.battlefield(P0, "Breaker of Creation");
    assert_eq!(keyword_count(&t, breaker, KeywordKind::Hexproof), 5);
    for spell in ["Swords to Plowshares", "Disfigure", "Lightning Bolt", "Giant Growth"] {
        assert!(!spell_can_target(&mut t, P1, spell, breaker), "{spell}");
    }
    // A colorless source's ability can target it.
    let rod = t.battlefield(P1, "Rod of Ruin");
    assert!(ability_can_target(&mut t, rod, 0, breaker));
}

#[test]
fn multiple_instances_of_hexproof_are_redundant() {
    cr!("702.11h");
    let mut t = TestGame::new(2);
    let bogle = t.battlefield(P0, "Slippery Bogle");
    t.lands(P0, "Forest", 2);
    let word = t.hand(P0, "Sheltering Word");
    t.cast(P0, word).target(bogle).go();
    t.resolve();
    assert_eq!(keyword_count(&t, bogle, KeywordKind::Hexproof), 2);
    // Still just hexproof: opponents can't target it, its controller can.
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", bogle));
    assert!(spell_can_target(&mut t, P0, "Giant Growth", bogle));
}
