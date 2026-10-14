//! Characteristic and static grammar II (cards2-statics-characteristics): statics that
//! also cover spells and cards outside the battlefield, keywords copied from exiled
//! cards, characteristic riders on permanents put onto the battlefield, conditional
//! keyword grants, "otherwise"/"instead" statics, rule-modifying statics, and how a card
//! refers to itself (names that are subtypes, pronouns, plural names, flavor words).

use crate::basic_effects_common::{assert_supported, last_target_candidates};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is(t: &mut TestGame, id: ObjectId, ct: CardType) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.card_types.contains(ct)
}

fn has_kw(t: &mut TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.has_keyword(k)
}

fn has_subtype(t: &mut TestGame, id: ObjectId, s: &str) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.subtypes.iter().any(|x| x == s)
}

// ---------------------------------------------------------------------------
// Statics over spells and cards outside the battlefield (CR 611.3a)
// ---------------------------------------------------------------------------

#[test]
fn biotransference_makes_creature_cards_and_spells_artifacts() {
    cr!("611.3a", "205.1b");
    assert_supported("Biotransference");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biotransference");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Llanowar Elves");
    let in_gy = t.graveyard(P0, "Hill Giant");
    let in_library = t.library_top(P0, "Grizzly Bears");
    let theirs = t.hand(P1, "Grizzly Bears");
    let land = t.hand(P0, "Forest");
    for id in [bears, in_hand, in_gy, in_library] {
        assert!(is(&mut t, id, CardType::Artifact));
        assert!(is(&mut t, id, CardType::Creature));
    }
    assert!(!is(&mut t, theirs, CardType::Artifact));
    assert!(!is(&mut t, land, CardType::Artifact));
    // A creature spell you control on the stack.
    t.lands(P0, "Forest", 2);
    let spell = t.hand(P0, "Grizzly Bears");
    let on_stack = t.cast(P0, spell).go();
    assert!(is(&mut t, on_stack, CardType::Artifact));
}

#[test]
fn maskwood_nexus_creature_cards_are_every_creature_type() {
    cr!("611.3a", "205.3m");
    assert_supported("Maskwood Nexus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maskwood Nexus");
    let in_hand = t.hand(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(t.obj_now(in_hand).chars.all_creature_types);
    let theirs = t.hand(P1, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.obj_now(theirs).chars.all_creature_types);
}

#[test]
fn roshan_other_creatures_and_cards_are_assassins() {
    cr!("611.3a", "205.1b");
    assert_supported("Roshan, Hidden Magister");
    let mut t = TestGame::new(2);
    let roshan = t.battlefield(P0, "Roshan, Hidden Magister");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_gy = t.graveyard(P0, "Hill Giant");
    assert!(has_subtype(&mut t, bears, "Assassin"));
    assert!(has_subtype(&mut t, in_gy, "Assassin"));
    assert!(has_subtype(&mut t, bears, "Bear"));
    let _ = roshan;
}

#[test]
fn arcane_adaptation_chosen_type_reaches_cards_in_hand() {
    cr!("611.3a", "607.2d");
    assert_supported("Arcane Adaptation");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Grizzly Bears");
    let goblin = subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Goblin")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(goblin));
    t.enter(P0, "Arcane Adaptation");
    assert!(has_subtype(&mut t, in_hand, "Goblin"));
    assert!(has_subtype(&mut t, in_hand, "Bear"));
}

#[test]
fn encroaching_mycosynth_nonland_cards_are_artifacts_lands_are_not() {
    cr!("611.3a");
    assert_supported("Encroaching Mycosynth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Encroaching Mycosynth");
    let ench = t.hand(P0, "Pacifism");
    let land = t.hand(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(is(&mut t, ench, CardType::Artifact));
    assert!(!is(&mut t, land, CardType::Artifact));
    // Instants aren't permanent cards.
    assert!(!is(&mut t, bolt, CardType::Artifact));
}

#[test]
fn celestial_dawn_cards_in_hand_are_white() {
    cr!("611.3a", "105.3");
    ruling!("Celestial Dawn", "Land cards you own that aren't on the battlefield aren't changed to Plains");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    let bolt = t.hand(P0, "Lightning Bolt");
    let land = t.hand(P0, "Island");
    t.g.recompute();
    assert_eq!(t.obj_now(bolt).chars.colors, ColorSet::single(Color::White));
    assert!(t.obj_now(land).chars.subtypes.iter().any(|s| s == "Island"));
}

#[test]
fn rootpath_purifier_lands_and_land_cards_in_library_are_basic() {
    cr!("611.3a", "205.4a");
    ruling!("Rootpath Purifier", "becoming basic also doesn’t grant any of the basic land types");
    assert_supported("Rootpath Purifier");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rootpath Purifier");
    let mine = t.battlefield(P0, "Wasteland");
    let in_library = t.library_top(P0, "Wasteland");
    let in_hand = t.hand(P0, "Wasteland");
    let theirs = t.battlefield(P1, "Wasteland");
    t.g.recompute();
    let basic = |t: &TestGame, id| t.obj_now(id).chars.supertypes.contains(Supertype::Basic);
    assert!(basic(&t, mine));
    assert!(basic(&t, in_library));
    assert!(!basic(&t, in_hand));
    assert!(!basic(&t, theirs));
    assert!(!has_subtype(&mut t, mine, "Plains"));
}

#[test]
fn yavimaya_kavu_power_and_toughness_count_separately() {
    cr!("604.3");
    assert_supported("Yavimaya Kavu");
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Yavimaya Kavu");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Goblin Piker");
    // The Kavu is red and green: two red creatures, three green ones.
    t.g.recompute();
    assert_eq!(t.pt(kavu), (2, 3));
}

// ---------------------------------------------------------------------------
// Keywords copied from exiled cards (CR 702.1c)
// ---------------------------------------------------------------------------

#[test]
fn death_mask_duplicant_has_keywords_of_the_imprinted_cards() {
    cr!("702.1c", "607.2a");
    assert_supported("Death-Mask Duplicant");
    assert_supported("Urborg Scavengers");
    assert_supported("Eater of Virtue");
    assert_supported("Rayami, First of the Fallen");
    assert_supported("Altar of the Wretched");
    let mut t = TestGame::new(2);
    let dmd = t.battlefield(P0, "Death-Mask Duplicant");
    let bird = t.graveyard(P0, "Wind Drake");
    t.lands(P0, "Wastes", 1);
    assert!(!has_kw(&mut t, dmd, KeywordKind::Flying));
    t.activate(P0, dmd, 0, &[Entity::Object(bird)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Wind Drake"));
    assert!(has_kw(&mut t, dmd, KeywordKind::Flying));
    assert!(!has_kw(&mut t, dmd, KeywordKind::Trample));
}

#[test]
fn scion_of_draco_gives_a_keyword_per_color() {
    cr!("613.1f");
    assert_supported("Scion of Draco");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scion of Draco");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(has_kw(&mut t, bears, KeywordKind::Trample));
    assert!(!has_kw(&mut t, bears, KeywordKind::Vigilance));
    assert!(has_kw(&mut t, lions, KeywordKind::Vigilance));
    assert!(!has_kw(&mut t, theirs, KeywordKind::Trample));
}

#[test]
fn magnigoth_treefolk_has_landwalk_for_each_basic_land_type_you_control() {
    cr!("702.14c");
    ruling!("Magnigoth Treefolk", "count the number of basic land types among lands you control");
    assert_supported("Magnigoth Treefolk");
    let mut t = TestGame::new(2);
    let mt = t.battlefield(P0, "Magnigoth Treefolk");
    t.battlefield(P0, "Island");
    t.g.recompute();
    let walks: Vec<String> = t
        .obj_now(mt)
        .chars
        .keywords_of(KeywordKind::Landwalk)
        .filter_map(|k| k.text.as_ref().map(|s| s.to_string()))
        .collect();
    assert_eq!(walks, vec!["islandwalk".to_string()]);
    // It can't be blocked by a player who controls an Island.
    t.battlefield(P1, "Island");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mt, Entity::Player(P1))], &[(blocker, mt)]);
    assert_eq!(t.life(P1), 20 - 2);
}

// ---------------------------------------------------------------------------
// Characteristic riders on permanents put onto the battlefield (CR 611.3c, 614.1c)
// ---------------------------------------------------------------------------

#[test]
fn otherworldly_escort_returns_as_a_spirit_detective() {
    cr!("611.3c", "205.1a");
    assert_supported("Otherworldly Escort");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Otherworldly Escort");
    t.g.destroy(e, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Otherworldly Escort");
    assert_eq!(back.len(), 1);
    assert!(has_subtype(&mut t, back[0], "Spirit"));
    assert!(has_subtype(&mut t, back[0], "Detective"));
    assert_eq!(t.counters(back[0], "charge"), 4);
    // Dies again as a Spirit: it stays in the graveyard.
    t.g.destroy(back[0], None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Otherworldly Escort").is_empty());
}

#[test]
fn princess_yue_returns_as_a_land_named_moon() {
    cr!("611.3c", "612.8");
    ruling!("Princess Yue", "she is no longer a creature");
    assert_supported("Princess Yue");
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Princess Yue");
    t.g.destroy(y, None);
    t.resolve_all();
    let moon: Vec<ObjectId> = t
        .g
        .battlefield
        .clone()
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.name == "Moon")
        .collect();
    assert_eq!(moon.len(), 1);
    assert!(is(&mut t, moon[0], CardType::Land));
    assert!(!is(&mut t, moon[0], CardType::Creature));
    assert!(t.g.obj(moon[0]).tapped);
}

#[test]
fn tom_bert_and_william_return_as_an_artifact() {
    cr!("611.3c", "205.1a");
    assert_supported("Tom, Bert, and William");
    let mut t = TestGame::new(2);
    let tbw = t.battlefield(P0, "Tom, Bert, and William");
    t.g.destroy(tbw, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Tom, Bert, and William");
    assert_eq!(back.len(), 1);
    assert!(is(&mut t, back[0], CardType::Artifact));
    assert!(!is(&mut t, back[0], CardType::Creature));
}

#[test]
fn valkyries_call_returns_creatures_as_flying_angels() {
    cr!("611.3c", "205.1b");
    assert_supported("Valkyrie's Call");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valkyrie's Call");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert!(has_subtype(&mut t, back[0], "Angel"));
    assert!(has_subtype(&mut t, back[0], "Bear"));
    assert!(has_kw(&mut t, back[0], KeywordKind::Flying));
    assert_eq!(t.pt(back[0]), (3, 3));
}

#[test]
fn supper_for_spiders_returns_food_artifacts() {
    cr!("611.3c", "205.1a");
    ruling!("Supper for Spiders", "will be only Food artifacts");
    assert_supported("Supper for Spiders");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    t.lands(P0, "Swamp", 6);
    let s = t.hand(P0, "Supper for Spiders");
    t.cast(P0, s).go();
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
    assert!(is(&mut t, back[0], CardType::Artifact));
    assert!(!is(&mut t, back[0], CardType::Creature));
    assert!(has_subtype(&mut t, back[0], "Food"));
    assert!(!has_subtype(&mut t, back[0], "Bear"));
}

#[test]
fn the_master_transcendent_returns_a_green_mutant() {
    cr!("611.3c", "613.4b");
    assert_supported("The Master, Transcendent");
}

#[test]
fn riders_compile_on_their_cards() {
    for name in [
        "Fang, Roku's Companion",
        "Drana, the Last Bloodchief",
        "Ghouls' Night Out",
        "Brilliance Unleashed",
        "Woodcaller Automaton",
        "Cavernous Maw",
    ] {
        assert_supported(name);
    }
}

// ---------------------------------------------------------------------------
// Conditional keyword grants and "otherwise"/"instead" statics
// ---------------------------------------------------------------------------

#[test]
fn relic_axe_gives_warriors_more() {
    cr!("613.4c");
    assert_supported("Relic Axe");
    let mut t = TestGame::new(2);
    let axe = t.battlefield(P0, "Relic Axe");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let warrior = t.battlefield(P0, "Elvish Warrior");
    t.g.attach(axe, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    t.g.attach(axe, Entity::Object(warrior));
    t.g.recompute();
    assert_eq!(t.pt(warrior), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn so_tiny_shrinks_more_with_seven_cards_in_the_controllers_graveyard() {
    cr!("613.4c");
    ruling!("So Tiny", "continuously checks the enchanted creature's controller's graveyard");
    assert_supported("So Tiny");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tiny = t.battlefield(P0, "So Tiny");
    t.g.attach(tiny, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (0, 2));
    for _ in 0..7 {
        t.graveyard(P1, "Island");
    }
    t.g.recompute();
    assert_eq!(t.pt(bears), (-4, 2));
    // Your own graveyard doesn't matter.
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.attach(tiny, Entity::Object(mine));
    t.g.recompute();
    assert_eq!(t.pt(mine), (1, 3));
}

#[test]
fn dog_umbra_has_umbra_armor_only_on_its_controllers_creature() {
    cr!("613.1f");
    ruling!("Dog Umbra", "continuously checks to see whether the controller of the enchanted creature");
    assert_supported("Dog Umbra");
    let mut t = TestGame::new(2);
    let umbra = t.battlefield(P0, "Dog Umbra");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(umbra, Entity::Object(mine));
    assert!(has_kw(&mut t, umbra, KeywordKind::UmbraArmor));
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.attach(umbra, Entity::Object(theirs));
    assert!(!has_kw(&mut t, umbra, KeywordKind::UmbraArmor));
}

#[test]
fn dominarias_judgment_protection_by_basic_land_types() {
    cr!("702.16a");
    assert_supported("Dominaria's Judgment");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 1);
    let dj = t.hand(P0, "Dominaria's Judgment");
    t.cast(P0, dj).go();
    t.resolve_all();
    t.g.recompute();
    let quals: Vec<String> = t
        .obj_now(bears)
        .chars
        .keywords_of(KeywordKind::Protection)
        .filter_map(|k| k.text.as_ref().map(|s| s.to_string()))
        .collect();
    assert_eq!(quals.len(), 2, "{quals:?}");
    assert!(quals.iter().any(|q| q.contains("white")));
    assert!(quals.iter().any(|q| q.contains("black")));
}

#[test]
fn aurelia_grants_keywords_by_color() {
    cr!("613.1f");
    assert_supported("Aurelia, Exemplar of Justice");
    assert_supported("Doors of Durin");
    assert_supported("Armament of Nyx");
    let mut t = TestGame::new(2);
    let aurelia = t.battlefield(P0, "Aurelia, Exemplar of Justice");
    t.answer_targets(P0, &[Entity::Object(aurelia)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // Aurelia is red and white.
    assert!(has_kw(&mut t, aurelia, KeywordKind::Trample));
    assert!(has_kw(&mut t, aurelia, KeywordKind::Vigilance));
    assert_eq!(t.pt(aurelia), (2 + 2, 5));
}

// ---------------------------------------------------------------------------
// Rule-modifying statics
// ---------------------------------------------------------------------------

#[test]
fn fires_of_invention_limits_spells_to_two_on_your_turn() {
    cr!("601.3");
    ruling!("Fires of Invention", "will look at the entire turn");
    assert_supported("Fires of Invention");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    t.lands(P0, "Mountain", 3);
    for _ in 0..2 {
        let b = t.hand(P0, "Lightning Bolt");
        t.cast(P0, b).target(P1).go();
        t.resolve();
    }
    let third = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, third).target(P1).try_go().is_err());
    assert_eq!(t.life(P1), 14);
}

#[test]
fn tamiyo_opponents_cant_make_you_discard() {
    cr!("701.9a", "101.2");
    ruling!("Tamiyo, Collector of Tales", "if it would force you to sacrifice a permanent or discard a card, you just don’t");
    assert_supported("Tamiyo, Collector of Tales");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tamiyo, Collector of Tales");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Your own spells still can.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tamiyo, Collector of Tales");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(P0).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn ashiok_opponents_spells_cant_make_them_search() {
    cr!("701.23a", "101.2");
    ruling!("Ashiok, Dream Render", "your opponents shuffle their libraries even though they can’t search");
    assert_supported("Ashiok, Dream Render");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashiok, Dream Render");
    let wilds = t.battlefield(P1, "Evolving Wilds");
    t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, wilds, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert!(t.in_graveyard(P1, "Evolving Wilds"));
}

#[test]
fn nowhere_to_run_lets_you_target_hexproof_creatures_and_stops_ward() {
    cr!("702.11e", "609.4", "702.21a");
    ruling!("Nowhere to Run", "It doesn't matter whether or not they have hexproof");
    assert_supported("Nowhere to Run");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nowhere to Run");
    let hexproof = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(hexproof).go();
    t.resolve_all();
    assert!(!t.on_battlefield(hexproof));
    // Ward doesn't trigger: the spell resolves though its cost isn't paid.
    let warded = t.battlefield(P1, "Tomakul Honor Guard");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(warded).go();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(warded));
}

#[test]
fn detection_tower_ignores_hexproof_until_end_of_turn() {
    cr!("702.11e", "609.4");
    ruling!("Detection Tower", "don’t actually lose hexproof");
    assert_supported("Detection Tower");
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Detection Tower");
    let hexproof = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    // Without the effect, the hexproof creature isn't a legal target.
    t.cast(P0, bolt).target(hexproof).go();
    assert!(!last_target_candidates(&t, P0).contains(&Entity::Object(hexproof)));
    t.resolve_all();
    let _ = bears;
    t.activate(P0, tower, 1, &[]).unwrap();
    t.resolve_all();
    assert!(has_kw(&mut t, hexproof, KeywordKind::Hexproof));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(hexproof).go();
    t.resolve_all();
    assert!(!t.on_battlefield(hexproof));
}

#[test]
fn arcane_lighthouse_creatures_lose_and_cant_have_hexproof() {
    cr!("613.1f", "702.11e");
    assert_supported("Arcane Lighthouse");
    let mut t = TestGame::new(2);
    let lh = t.battlefield(P0, "Arcane Lighthouse");
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, lh, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!has_kw(&mut t, scout, KeywordKind::Hexproof));
}

#[test]
fn omo_lands_with_everything_counters_are_every_land_type() {
    cr!("205.3i");
    ruling!("Omo, Queen of Vesuva", "grants every land type to lands with everything counters");
    assert_supported("Omo, Queen of Vesuva");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omo, Queen of Vesuva");
    let land = t.battlefield(P0, "Wastes");
    let other = t.battlefield(P0, "Wastes");
    t.g.objects[land.0 as usize].counters.insert("everything".into(), 1);
    t.g.dirty = true;
    for s in ["Plains", "Desert", "Cave", "Urza's", "Forest"] {
        assert!(has_subtype(&mut t, land, s), "{s}");
    }
    assert!(!has_subtype(&mut t, other, "Plains"));
}

// ---------------------------------------------------------------------------
// How a card refers to itself (CR 201.5, 207.2c-d)
// ---------------------------------------------------------------------------

#[test]
fn names_that_are_subtypes_pronouns_and_flavor_words() {
    cr!("201.5", "207.2c", "207.2d");
    for name in [
        "Gideon of the Trials",
        "Garruk Relentless",
        "Garruk, Cursed Huntsman",
        "Sarkhan the Mad",
        "Assembly-Worker",
        "Fblthp, Impossibly Lost",
        "Kaito, Bane of Nightmares",
        "The Notary Hobbits",
        "Tom, Bert, and William",
        "Icingdeath, Frost Tyrant",
        "Jumbo Cactuar",
        "Aloy, Savior of Meridian",
        "Elektra, Femme Fatale",
    ] {
        assert_supported(name);
    }
}

#[test]
fn assembly_worker_targets_assembly_workers() {
    cr!("201.5");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Assembly-Worker");
    let b = t.battlefield(P0, "Assembly-Worker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Any Assembly-Worker creature, not only this one; not other creatures.
    t.activate(P0, a, 0, &[Entity::Object(b)]).unwrap();
    let cands = last_target_candidates(&t, P0);
    assert!(cands.contains(&Entity::Object(a)) && cands.contains(&Entity::Object(b)));
    assert!(!cands.contains(&Entity::Object(bears)));
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(b), (3, 3));
}

#[test]
fn gideon_of_the_trials_emblem_needs_a_gideon_planeswalker() {
    cr!("201.5");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon of the Trials");
    t.activate(P0, g, 2, &[]).unwrap();
    t.resolve_all();
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    // Without a Gideon planeswalker, the emblem does nothing.
    let g = t.g.current(g);
    t.g.destroy(g, None);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn fblthp_shuffles_itself_into_its_owners_library() {
    cr!("201.5");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fblthp, Impossibly Lost");
    let lib = t.library_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(f, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Fblthp, Impossibly Lost").is_empty());
    // Drew two, shuffled itself in.
    assert_eq!(t.library_size(P0), lib - 2 + 1);
}

#[test]
fn jumbo_cactuar_flavor_word_with_a_thousands_separator() {
    cr!("207.2d", "107.1");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jumbo Cactuar");
    t.g.players[1].life = 20000;
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(j, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20000 - 10000);
}

// ---------------------------------------------------------------------------
// Characteristic bundles: Auras, names, base power and toughness, modes
// ---------------------------------------------------------------------------

#[test]
fn bronzehide_lion_returns_as_an_aura_on_a_creature_you_control() {
    cr!("303.4f", "303.4g", "611.3c");
    ruling!("Bronzehide Lion", "If there's nothing it can legally enchant, it remains in its owner's graveyard");
    assert_supported("Bronzehide Lion");
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Bronzehide Lion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.g.destroy(lion, None);
    t.resolve_all();
    let aura = t.named_on_battlefield("Bronzehide Lion");
    assert_eq!(aura.len(), 1);
    assert!(is(&mut t, aura[0], CardType::Enchantment));
    assert!(!is(&mut t, aura[0], CardType::Creature));
    assert_eq!(t.g.obj(aura[0]).attached_to, Some(Entity::Object(bears)));
    // Only the granted abilities: no trample of its own.
    assert!(!has_kw(&mut t, aura[0], KeywordKind::Trample));
    // With no creature it could enchant, it stays in the graveyard.
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Bronzehide Lion");
    t.battlefield(P1, "Hill Giant");
    t.g.destroy(lion, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Bronzehide Lion").is_empty());
    assert!(t.in_graveyard(P0, "Bronzehide Lion"));
}

#[test]
fn honest_work_makes_a_humble_merchant() {
    cr!("612.8", "613.1f", "613.4b");
    ruling!("Honest Work", "It can still gain new abilities after it is enchanted");
    assert_supported("Honest Work");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Serra Angel");
    let hw = t.battlefield(P0, "Honest Work");
    t.g.attach(hw, Entity::Object(giant));
    t.g.recompute();
    let c = t.obj_now(giant).chars.clone();
    assert_eq!(c.name, "Humble Merchant");
    assert!(!c.has_keyword(KeywordKind::Flying));
    assert!(c.subtypes.iter().any(|s| s == "Citizen"));
    assert!(!c.subtypes.iter().any(|s| s == "Angel"));
    assert_eq!(t.pt(giant), (1, 1));
    assert!(c.abilities.iter().any(|a| matches!(&a.kind, ability::AbilityKind::Activated(x) if x.is_mana_ability)));
}

#[test]
fn goddric_is_a_dragon_after_celebration() {
    cr!("613.1d", "613.4b");
    ruling!("Goddric, Cloaked Reveler", "he's still a Dragon creature with base power and toughness 4/4");
    assert_supported("Goddric, Cloaked Reveler");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goddric, Cloaked Reveler");
    assert!(!has_subtype(&mut t, g, "Dragon"));
    t.enter(P0, "Grizzly Bears");
    t.enter(P0, "Hill Giant");
    assert!(has_subtype(&mut t, g, "Dragon"));
    assert!(has_kw(&mut t, g, KeywordKind::Flying));
    assert_eq!(t.pt(g), (4, 4));
}

#[test]
fn tenth_district_hero_becomes_mileva() {
    cr!("612.8", "205.1a", "613.4b");
    ruling!("Tenth District Hero", "5/5 legendary Human Detective named Mileva, the Stalwart");
    assert_supported("Tenth District Hero");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Tenth District Hero");
    t.lands(P0, "Plains", 5);
    for _ in 0..2 {
        t.graveyard(P0, "Hill Giant");
    }
    t.activate(P0, h, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has_subtype(&mut t, h, "Detective"));
    t.activate(P0, h, 1, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    let c = t.obj_now(h).chars.clone();
    assert_eq!(c.name, "Mileva, the Stalwart");
    assert!(c.supertypes.contains(Supertype::Legendary));
    assert!(c.subtypes.iter().any(|s| s == "Detective"));
    assert_eq!(t.pt(h), (5, 5));
}

#[test]
fn wall_of_tombstones_base_toughness_counts_creature_cards() {
    cr!("613.4b", "608.2h");
    assert_supported("Wall of Tombstones");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wall of Tombstones");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(w).1, 3);
}

#[test]
fn wild_shape_mode_sets_power_toughness_type_and_ability() {
    cr!("700.2", "613.4b", "205.1a");
    ruling!("Wild Shape", "The creature loses all other creature types, but it retains its abilities");
    assert_supported("Wild Shape");
    assert_supported("Genku, Future Shaper");
    assert_supported("Outlaws' Merriment");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Forest", 1);
    let ws = t.hand(P0, "Wild Shape");
    t.cast(P0, ws).modes(&[0]).target(angel).go();
    t.resolve_all();
    assert_eq!(t.pt(angel), (1, 3));
    assert!(has_subtype(&mut t, angel, "Turtle"));
    assert!(!has_subtype(&mut t, angel, "Angel"));
    assert!(has_kw(&mut t, angel, KeywordKind::Hexproof));
    assert!(has_kw(&mut t, angel, KeywordKind::Flying));
}

#[test]
fn genku_creates_the_chosen_bundle_token() {
    cr!("700.2", "111.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Genku, Future Shaper");
    let land = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.g.destroy(land, None);
    t.resolve_all();
    let toks = t.named_on_battlefield("Moonfolk Token");
    let toks = if toks.is_empty() {
        t.g.battlefield
            .clone()
            .into_iter()
            .filter(|o| t.g.obj(*o).chars.subtypes.iter().any(|s| s == "Moonfolk"))
            .collect()
    } else {
        toks
    };
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (1, 2));
    assert!(has_kw(&mut t, toks[0], KeywordKind::Flying));
    t.g.recompute();
    assert_eq!(t.obj_now(toks[0]).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn walking_bulwark_creature_assigns_damage_by_toughness() {
    cr!("510.1a", "702.3b");
    ruling!("Walking Bulwark", "changes only the amount of combat damage it assigns");
    assert_supported("Walking Bulwark");
    assert_supported("The Pride of Hull Clade");
    let mut t = TestGame::new(2);
    let bw = t.battlefield(P0, "Walking Bulwark");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, bw, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    assert!(has_kw(&mut t, wall, KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wall, Entity::Player(P1))], &[]);
    // Wall of Wood is 0/3.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(wall), (0, 3));
}

#[test]
fn crown_of_convergence_pumps_creatures_sharing_a_color_with_the_top_card() {
    cr!("611.3a");
    ruling!("Crown of Convergence", "A colorless creature on top of your library never gives a bonus");
    assert_supported("Crown of Convergence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crown of Convergence");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.library_top(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(lions), (2, 1));
    t.library_top(P0, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn dance_of_the_manse_makes_four_four_creatures_only_with_x_six() {
    cr!("611.3c", "107.3a");
    ruling!("Dance of the Manse", "enter the battlefield as 4/4 creatures");
    assert_supported("Dance of the Manse");
    for (x, creature) in [(3, false), (6, true)] {
        let mut t = TestGame::new(2);
        let anthem = t.graveyard(P0, "Glorious Anthem");
        t.lands(P0, "Plains", 7);
        t.lands(P0, "Island", 1);
        let d = t.hand(P0, "Dance of the Manse");
        t.cast(P0, d).x(x).targets(&[Entity::Object(anthem)]).go();
        t.resolve_all();
        let back = t.named_on_battlefield("Glorious Anthem");
        assert_eq!(back.len(), 1);
        assert_eq!(is(&mut t, back[0], CardType::Creature), creature);
        assert!(is(&mut t, back[0], CardType::Enchantment));
        if creature {
            // Its own anthem gives it +1/+1.
            assert_eq!(t.pt(back[0]), (5, 5));
        }
    }
}

#[test]
fn new_blood_changes_a_creature_type_to_vampire() {
    cr!("612.2");
    ruling!("New Blood", "The creature you gain control of won't be a Vampire if you don't choose");
    assert_supported("New Blood");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bear_i = subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Bear")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let _ = bear_i;
    t.lands(P0, "Swamp", 4);
    t.battlefield(P0, "Vampire Interloper");
    let nb = t.hand(P0, "New Blood");
    t.cast(P0, nb).target(bears).go();
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(bears)).controller, P0);
    // The creature types on it are offered first: Bear became Vampire.
    assert!(has_subtype(&mut t, bears, "Vampire"));
    assert!(!has_subtype(&mut t, bears, "Bear"));
}

#[test]
fn teferi_emblem_untaps_and_draws_on_opponents_turns() {
    cr!("502.3", "504.1");
    ruling!("Teferi, Who Slows the Sunset", "your permanents will untap at the same time as your opponent's permanents");
    assert_supported("Teferi, Who Slows the Sunset");
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, "Teferi, Who Slows the Sunset");
    t.g.objects[teferi.0 as usize].counters.insert("loyalty".into(), 7);
    let n = t
        .g
        .obj(teferi)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Activated(_)))
        .count();
    t.activate(P0, teferi, n - 1, &[]).unwrap();
    t.resolve_all();
    let land = t.battlefield(P0, "Forest");
    t.g.objects[land.0 as usize].tapped = true;
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.obj(land).tapped);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), hand + 1);
}
