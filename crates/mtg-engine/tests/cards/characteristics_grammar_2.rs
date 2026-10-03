//! Characteristic-changing grammar, continued: animation with "still a [type]", "as
//! [objects] enter, it becomes ...", base power and toughness taken from other objects,
//! P/T switching, characteristic-defining power and toughness from life totals, and
//! protection qualities taken from the game (CR 205.1b, 604.3, 611.2, 613.4, 702.16).

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn chars(t: &TestGame, id: ObjectId) -> &object::Characteristics {
    &t.obj_now(id).chars
}

/// P0 attacks P1 with `attackers` and stops once the declare attackers step's triggers
/// are on the stack.
fn attack_with(t: &mut TestGame, attackers: &[ObjectId]) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.iter().map(|a| (*a, Entity::Player(P1))).collect()),
    );
    t.advance_to(P0, Step::DeclareAttackers);
}

#[test]
fn gideon_blackblade_is_a_creature_only_during_your_turn() {
    cr!("205.1b", "306.4");
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    t.advance_to(P0, Step::PrecombatMain);
    let c = chars(&t, gideon);
    assert!(c.is_creature());
    assert!(c.card_types.contains(CardType::Planeswalker));
    assert!(c.has_subtype("Human") && c.has_subtype("Soldier") && c.has_subtype("Gideon"));
    assert!(c.has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.pt(gideon), (4, 4));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!chars(&t, gideon).is_creature());
}

#[test]
fn haunted_plate_mail_becomes_a_spirit_that_is_no_longer_an_equipment() {
    cr!("205.1b", "613.1d");
    ruling!("Haunted Plate Mail", "the ability will have no effect");
    assert_supported("Haunted Plate Mail");
    let mut t = TestGame::new(2);
    let mail = t.battlefield(P0, "Haunted Plate Mail");
    t.activate(P0, mail, 0, &[]).unwrap();
    t.resolve();
    let c = chars(&t, mail);
    assert!(c.is_creature());
    assert!(c.card_types.contains(CardType::Artifact));
    assert!(c.has_subtype("Spirit"));
    assert!(!c.has_subtype("Equipment"));
    assert_eq!(t.pt(mail), (4, 4));
}

#[test]
fn cacophony_unleashed_is_still_an_enchantment_while_animated() {
    cr!("205.1b", "611.2a");
    assert_supported("Cacophony Unleashed");
    let mut t = TestGame::new(2);
    let cacophony = t.battlefield(P0, "Cacophony Unleashed");
    t.enter(P0, "Earnest Fellowship");
    t.resolve_all();
    let c = chars(&t, cacophony);
    assert!(c.is_creature());
    assert!(c.card_types.contains(CardType::Enchantment));
    assert!(c.is_legendary());
    assert!(c.has_subtype("Nightmare") && c.has_subtype("God"));
    assert!(c.has_keyword(KeywordKind::Menace) && c.has_keyword(KeywordKind::Deathtouch));
    assert_eq!(t.pt(cacophony), (6, 6));
    t.advance_to(P1, Step::Upkeep);
    assert!(!chars(&t, cacophony).is_creature());
}

#[test]
fn displaced_dinosaurs_makes_historic_permanents_enter_as_dinosaurs() {
    cr!("614.1c", "700.6");
    ruling!("Displaced Dinosaurs", "it doesn't enter and then become");
    assert_supported("Displaced Dinosaurs");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Displaced Dinosaurs");
    let rock = t.enter(P0, "Mind Stone");
    let bear = t.enter(P0, "Grizzly Bears");
    let theirs = t.enter(P1, "Mind Stone");
    let c = chars(&t, rock);
    assert!(c.is_creature() && c.card_types.contains(CardType::Artifact));
    assert!(c.has_subtype("Dinosaur"));
    assert_eq!(t.pt(rock), (7, 7));
    assert_eq!(t.pt(bear), (2, 2));
    assert!(!chars(&t, theirs).is_creature());
}

#[test]
fn narfi_pumps_creatures_that_are_snow_or_zombies() {
    cr!("205.4a", "613.4c");
    ruling!("Narfi, Betrayer King", "will get only +1/+1");
    assert_supported("Narfi, Betrayer King");
    let mut t = TestGame::new(2);
    let narfi = t.battlefield(P0, "Narfi, Betrayer King");
    let troll = t.battlefield(P0, "Icehide Troll");
    let corpse = t.battlefield(P0, "Walking Corpse");
    let bear = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(troll), (3, 4));
    assert_eq!(t.pt(corpse), (3, 3));
    assert_eq!(t.pt(bear), (2, 2));
    // "Other": Narfi (a snow Zombie) doesn't pump itself.
    assert_eq!(t.pt(narfi), (4, 3));
}

#[test]
fn glistening_deluge_gives_green_and_white_creatures_an_additional_minus_two() {
    cr!("611.2c", "613.4c");
    assert_supported("Glistening Deluge");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let deluge = t.hand(P0, "Glistening Deluge");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, deluge).go();
    t.resolve();
    assert_eq!(t.pt(wurm), (3, 1));
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn valakut_fireboar_switches_when_it_attacks() {
    cr!("613.4d");
    ruling!("Valakut Fireboar", "apply after all other effects");
    assert_supported("Valakut Fireboar");
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Valakut Fireboar");
    attack_with(&mut t, &[boar]);
    t.resolve_all();
    assert_eq!(t.pt(boar), (7, 1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn exuberant_wolfbear_and_unruly_krasis_set_another_creatures_base_pt() {
    cr!("613.4b", "608.2h");
    assert_supported("Exuberant Wolfbear");
    assert_supported("Unruly Krasis");
    let mut t = TestGame::new(2);
    let wolfbear = t.battlefield(P0, "Exuberant Wolfbear");
    let human = t.battlefield(P0, "Elite Vanguard");
    t.answer_targets(P0, &[Entity::Object(human)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[wolfbear]);
    t.resolve_all();
    assert_eq!(t.pt(human), (4, 4));

    let mut t = TestGame::new(2);
    let krasis = t.battlefield(P0, "Unruly Krasis");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[krasis]);
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn lithomantic_barrage_deals_five_to_a_white_or_blue_target() {
    cr!("608.2c");
    assert_supported("Lithomantic Barrage");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let a = t.hand(P0, "Lithomantic Barrage");
    let b = t.hand(P0, "Lithomantic Barrage");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, a).target(angel).go();
    t.resolve();
    t.cast(P0, b).target(wurm).go();
    t.resolve();
    assert!(!t.on_battlefield(angel));
    assert_eq!(t.obj_now(wurm).damage, 1);
}

#[test]
fn bladed_bracers_gives_vigilance_only_to_humans_and_angels() {
    cr!("301.5", "604.2");
    assert_supported("Bladed Bracers");
    let mut t = TestGame::new(2);
    let bracers = t.battlefield(P0, "Bladed Bracers");
    let human = t.battlefield(P0, "Elite Vanguard");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(bracers, Entity::Object(bear));
    t.g.recompute();
    assert!(!chars(&t, bear).has_keyword(KeywordKind::Vigilance));
    assert_eq!(t.pt(bear), (3, 3));
    t.g.attach(bracers, Entity::Object(human));
    t.g.recompute();
    assert!(chars(&t, human).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn alacrian_armory_animates_a_vehicle_at_the_beginning_of_combat() {
    cr!("603.2", "301.7b");
    assert_supported("Alacrian Armory");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alacrian Armory");
    let car = t.battlefield(P0, "Smuggler's Copter");
    t.answer_targets(P0, &[Entity::Object(car)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let c = chars(&t, car);
    assert!(c.is_creature());
    assert!(c.has_keyword(KeywordKind::Vigilance));
    assert_eq!(t.pt(car), (3, 4));
}

#[test]
fn the_archimandrite_pumps_advisors_artificers_and_monks() {
    cr!("603.2", "613.4c");
    assert_supported("The Archimandrite");
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "The Archimandrite");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.gain_life(P0, 3);
    t.resolve_all();
    assert_eq!(t.pt(arch), (3, 5));
    assert!(chars(&t, arch).has_keyword(KeywordKind::Vigilance));
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn phyrexian_ingester_gets_the_exiled_creatures_power_and_toughness() {
    cr!("607.2a", "613.4c");
    assert_supported("Phyrexian Ingester");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let ingester = t.hand(P0, "Phyrexian Ingester");
    t.lands(P0, "Island", 7);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer_yes(P0, true);
    t.cast(P0, ingester).go();
    t.resolve_all();
    let ingester = t.named_on_battlefield("Phyrexian Ingester")[0];
    assert!(t.in_exile("Craw Wurm"));
    assert_eq!(t.pt(ingester), (9, 7));
}

#[test]
fn wishmonger_protection_color_is_chosen_by_the_targets_controller() {
    cr!("608.2d", "702.16b");
    assert_supported("Wishmonger");
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Wishmonger");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, monger, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve();
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, decision::Decision::ChooseOption { .. })));
    assert!(chars(&t, bear).has_keyword(KeywordKind::Protection));
}

#[test]
fn shadow_puppeteers_can_make_an_attacking_flyer_a_red_dragon() {
    cr!("613.4b", "105.3", "205.1b");
    assert_supported("Shadow Puppeteers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let bird = t.battlefield(P0, "Wind Drake");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[bird]);
    t.resolve_all();
    let c = chars(&t, bird);
    assert!(c.has_subtype("Dragon") && c.has_subtype("Drake"));
    assert!(c.colors.contains(Color::Red) && c.colors.contains(Color::Blue));
    assert_eq!(t.pt(bird), (4, 4));
}

#[test]
fn opal_titan_gets_protection_from_the_spells_colors() {
    cr!("603.4", "702.16b");
    assert_supported("Opal Titan");
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Opal Titan");
    let bear = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    t.advance_to(P1, Step::PrecombatMain);
    t.cast(P1, bear).go();
    t.resolve_all();
    let c = chars(&t, titan);
    assert!(c.is_creature() && !c.card_types.contains(CardType::Enchantment));
    assert!(c.has_subtype("Giant"));
    assert_eq!(t.pt(titan), (4, 4));
    let green = t.hand(P1, "Giant Growth");
    let red = t.hand(P1, "Shock");
    assert!(t.g.object_untargetable(titan, P1, Some(green)));
    assert!(!t.g.object_untargetable(titan, P1, Some(red)));
}

#[test]
fn energybending_gives_lands_every_basic_land_type() {
    cr!("305.6", "205.1b");
    assert_supported("Energybending");
    let mut t = TestGame::new(2);
    let wastes = t.battlefield(P0, "Wastes");
    let spell = t.hand(P0, "Energybending");
    t.lands(P0, "Plains", 2);
    t.cast(P0, spell).go();
    t.resolve();
    for st in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(chars(&t, wastes).has_subtype(st), "{st}");
    }
}

#[test]
fn zur_animates_an_enchantment_with_its_mana_value() {
    cr!("613.4b", "608.2h");
    assert_supported("Zur, Eternal Schemer");
    let mut t = TestGame::new(2);
    let zur = t.battlefield(P0, "Zur, Eternal Schemer");
    let ench = t.battlefield(P0, "Earnest Fellowship");
    t.lands(P0, "Plains", 2);
    t.activate(P0, zur, 0, &[Entity::Object(ench)]).unwrap();
    t.resolve();
    let c = chars(&t, ench);
    assert!(c.is_creature() && c.card_types.contains(CardType::Enchantment));
    assert_eq!(t.pt(ench), (2, 2));
    // Zur gives enchantment creatures you control deathtouch.
    assert!(c.has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn amplifire_doubles_the_revealed_creatures_power_and_toughness() {
    cr!("613.4b", "608.2h");
    assert_supported("Amplifire");
    let mut t = TestGame::new(2);
    let fire = t.battlefield(P0, "Amplifire");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Shock");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.pt(fire), (12, 8));
}

#[test]
fn empty_shrine_kannushi_has_protection_from_colors_among_your_permanents() {
    cr!("702.16b", "611.3a");
    assert_supported("Empty-Shrine Kannushi");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Empty-Shrine Kannushi");
    let bolt = t.hand(P1, "Lightning Bolt");
    let growth = t.hand(P1, "Giant Growth");
    // It is white itself.
    let swords = t.hand(P1, "Swords to Plowshares");
    assert!(t.g.object_untargetable(k, P1, Some(swords)));
    assert!(!t.g.object_untargetable(k, P1, Some(bolt)));
    t.battlefield(P0, "Raging Goblin");
    t.g.recompute();
    assert!(t.g.object_untargetable(k, P1, Some(bolt)));
    assert!(!t.g.object_untargetable(k, P1, Some(growth)));
}

#[test]
fn roiling_horror_and_scourge_of_the_skyclaves_read_life_totals() {
    cr!("604.3", "613.4a");
    ruling!("Roiling Horror", "uses whichever value is the largest");
    assert_supported("Scourge of the Skyclaves");
    let mut t = TestGame::with_config(3, Default::default());
    let horror = t.battlefield(P0, "Roiling Horror");
    t.g.players[0].life = 25;
    t.g.players[1].life = 18;
    t.g.players[2].life = 20;
    t.g.recompute();
    assert_eq!(t.pt(horror), (5, 5));
    let scourge = t.battlefield(P0, "Scourge of the Skyclaves");
    t.g.players[0].life = 12;
    t.g.players[1].life = 9;
    t.g.players[2].life = 7;
    t.g.recompute();
    assert_eq!(t.pt(scourge), (8, 8));
}

#[test]
fn jump_scare_makes_a_flying_horror_enchantment_creature() {
    cr!("205.1b", "613.4c");
    assert_supported("Jump Scare");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Jump Scare");
    t.lands(P0, "Plains", 1);
    t.cast(P0, spell).target(bear).go();
    t.resolve();
    let c = chars(&t, bear);
    assert!(c.card_types.contains(CardType::Enchantment) && c.is_creature());
    assert!(c.has_subtype("Horror") && c.has_subtype("Bear"));
    assert!(c.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(bear), (4, 4));
}

#[test]
fn fractalize_sets_colors_types_and_base_pt_from_x() {
    cr!("613.4b", "105.3", "205.1a");
    assert_supported("Fractalize");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let spell = t.hand(P0, "Fractalize");
    t.lands(P0, "Island", 4);
    t.cast(P0, spell).x(3).target(wurm).go();
    t.resolve();
    let c = chars(&t, wurm);
    assert!(c.has_subtype("Fractal") && !c.has_subtype("Wurm"));
    assert!(c.colors.contains(Color::Green) && c.colors.contains(Color::Blue));
    assert_eq!(c.colors.count(), 2);
    assert_eq!(t.pt(wurm), (4, 4));
}

#[test]
fn sarkhan_becomes_a_flying_dragon_when_a_dragon_enters() {
    cr!("205.1b", "603.2");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Dragon Ascendant");
    t.enter(P0, "Shivan Dragon");
    t.resolve_all();
    let c = chars(&t, sarkhan);
    assert!(c.has_subtype("Dragon") && c.has_subtype("Human"));
    assert!(c.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(sarkhan), (3, 3));
}

#[test]
fn ursine_champion_and_wishful_merfolk_replace_their_creature_types() {
    cr!("205.1a", "613.1d");
    assert_supported("Ursine Champion");
    assert_supported("Wishful Merfolk");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Ursine Champion");
    let merfolk = t.battlefield(P0, "Wishful Merfolk");
    t.lands(P0, "Forest", 6);
    t.lands(P0, "Island", 2);
    t.activate(P0, champ, 0, &[]).unwrap();
    t.resolve();
    let c = chars(&t, champ);
    assert!(c.has_subtype("Bear") && c.has_subtype("Berserker") && !c.has_subtype("Human"));
    assert_eq!(t.pt(champ), (5, 5));
    t.activate(P0, merfolk, 0, &[]).unwrap();
    t.resolve();
    let c = chars(&t, merfolk);
    assert!(c.has_subtype("Human") && !c.has_subtype("Merfolk"));
    assert!(!c.has_keyword(KeywordKind::Defender));
}

#[test]
fn bogardan_dragonheart_becomes_a_hasty_flying_dragon() {
    cr!("613.4b", "205.1a");
    assert_supported("Bogardan Dragonheart");
    let mut t = TestGame::new(2);
    let heart = t.battlefield(P0, "Bogardan Dragonheart");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.activate(P0, heart, 0, &[]).unwrap();
    t.resolve();
    let c = chars(&t, heart);
    assert!(c.has_subtype("Dragon") && !c.has_subtype("Human"));
    assert!(c.has_keyword(KeywordKind::Flying) && c.has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(heart), (4, 4));
}
