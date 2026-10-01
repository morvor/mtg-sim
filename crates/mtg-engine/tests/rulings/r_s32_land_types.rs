//! Rulings batch S32 — land types (CR 205.3i, 305.6–305.8): basic land types give a land
//! its mana ability, other land types (Town, Desert, Gate, Lair) mean nothing by
//! themselves; "basic" is a supertype (CR 205.4); type-changing effects leave supertypes
//! alone (CR 305.7); lands that become creatures keep everything else (CR 305.9, 613.1d).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s27_common::can_activate_containing;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s25_common::cast_new;
use crate::r_s32_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of every choice of objects asked since decision `from`.
fn all_candidates(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .flat_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => candidates.clone(),
            Decision::ChooseTargets { .. } => vec![],
            _ => vec![],
        })
        .collect()
}

/// The mana `p` gets by tapping `land` with its ability whose text contains `needle`.
fn mana_from(t: &mut TestGame, p: PlayerId, land: ObjectId, needle: &str) -> Vec<ManaType> {
    t.g.players[p.idx()].mana_pool = Default::default();
    assert!(tap_for_mana(t, p, land, needle), "couldn't tap for {needle}");
    ManaType::ALL
        .iter()
        .flat_map(|ty| std::iter::repeat_n(*ty, pool(t, p, *ty) as usize))
        .collect()
}

#[test]
fn town_is_a_land_type_with_no_intrinsic_abilities() {
    cr!("205.3i", "305.6", "305.8");
    ruling!(
        "The Gold Saucer",
        "Town is a land type with no special meaning. It doesn't grant the land any intrinsic abilities. Other cards may care about which lands are Towns."
    );
    supported("The Gold Saucer");
    let mut t = TestGame::new(2);
    let saucer = t.battlefield(P0, "The Gold Saucer");
    assert_eq!(subtypes_now(&t, saucer), vec!["Town"]);
    assert!(!basic_now(&t, saucer));
    // Its three printed abilities, and no others: it taps only for {C}.
    assert_eq!(t.obj(saucer).chars.abilities.len(), 3);
    assert_eq!(mana_from(&mut t, P0, saucer, "Add {C}"), vec![ManaType::C]);
    // A Forest's basic land type gives it "{T}: Add {G}." (CR 305.6).
    let forest = t.battlefield(P0, "Forest");
    assert_eq!(mana_from(&mut t, P0, forest, "{G}"), vec![ManaType::G]);
}

#[test]
fn desert_is_a_land_type_with_no_intrinsic_mana_ability() {
    cr!("205.3i", "305.6", "603.4", "506.4", "602.5b");
    ruling!(
        "Sunscorched Desert",
        "Desert is a land subtype with no special meaning. It doesn’t grant the land an intrinsic mana ability. Other cards may care about which lands are Deserts."
    );
    ruling!(
        "Desert",
        "Desert is a land subtype with no special meaning. It doesn't grant the land an intrinsic mana ability. Other cards may care about which lands are Deserts."
    );
    supported("Sunscorched Desert");
    supported("Desert");
    supported("Sand Strangler");
    let mut t = TestGame::new(2);
    let desert = t.battlefield(P0, "Desert");
    assert_eq!(subtypes_now(&t, desert), vec!["Desert"]);
    // Only its printed mana ability ({C}) and its damage ability.
    assert_eq!(t.obj(desert).chars.abilities.len(), 2);
    assert_eq!(mana_from(&mut t, P0, desert, "Add {C}"), vec![ManaType::C]);
    // Its other ability: "{T}: This land deals 1 damage to target attacking creature.
    // Activate only during the end of combat step."
    let desert = t.battlefield(P0, "Desert");
    assert!(
        !can_activate_containing(&mut t, P0, desert, "damage"),
        "not in a main phase"
    );
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    assert!(
        !can_activate_containing(&mut t, P0, desert, "damage"),
        "not in the declare attackers step"
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert!(can_activate_containing(&mut t, P0, desert, "damage"));
    t.activate(P0, desert, 1, &[Entity::Object(bears)])
        .expect("activate during the end of combat step");
    t.resolve_all();
    assert_eq!(t.obj(bears).damage, 1);
    let mut t = TestGame::new(2);
    let sunscorched = t.battlefield(P0, "Sunscorched Desert");
    assert_eq!(t.obj(sunscorched).chars.abilities.len(), 2);
    assert_eq!(
        mana_from(&mut t, P0, sunscorched, "Add {C}"),
        vec![ManaType::C]
    );
    // Sand Strangler cares: "When this creature enters, if you control a Desert or there
    // is a Desert card in your graveyard, you may have this creature deal 3 damage to
    // target creature."
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Sand Strangler");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"), "Sunscorched Desert is a Desert");
}

#[test]
fn a_land_with_two_basic_land_types_isnt_basic() {
    cr!("205.4c", "305.8", "701.23a");
    ruling!(
        "Watery Grave",
        "Unlike most dual lands, this land has two basic land types. It's not basic, so cards such as District Guide can't find it, but it does have the appropriate land types for effects such as that of Drowned Catacomb (from the Ixalan set)."
    );
    ruling!(
        "Hallowed Fountain",
        "This land has two basic land types. It's not basic, so cards such as Tend the Sprigs can't find it, but it does have the appropriate land types for effects such as that of Kishla Village (from the Tarkir: Dragonstorm release)."
    );
    supported("Watery Grave");
    supported("District Guide");
    supported("Drowned Catacomb");
    supported("Tend the Sprigs");
    supported("Kishla Village");
    // District Guide: "you may search your library for a basic land card or Gate card".
    let mut t = TestGame::new(2);
    let grave = t.library_top(P0, "Watery Grave");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.enter(P0, "District Guide");
    t.resolve_all();
    assert!(!all_candidates(&t, from).contains(&Entity::Object(grave)));
    assert!(!t.in_hand(P0, "Watery Grave"));
    // Drowned Catacomb: "This land enters tapped unless you control an Island or a
    // Swamp."
    let mut t = TestGame::new(2);
    let grave = t.battlefield(P0, "Watery Grave");
    assert!(!basic_now(&t, grave));
    assert_eq!(subtypes_now(&t, grave), vec!["Island", "Swamp"]);
    let catacomb = t.enter(P0, "Drowned Catacomb");
    assert!(!t.obj(catacomb).tapped);
    // Tend the Sprigs: "Search your library for a basic land card".
    let mut t = TestGame::new(2);
    let fountain = t.library_top(P0, "Hallowed Fountain");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Tend the Sprigs", &[]);
    t.resolve_all();
    assert!(!all_candidates(&t, from).contains(&Entity::Object(fountain)));
    assert!(t.named_on_battlefield("Hallowed Fountain").is_empty());
    // Kishla Village: "This land enters tapped unless you control an Island or a Swamp."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hallowed Fountain");
    let village = t.enter(P0, "Kishla Village");
    assert!(!t.obj(village).tapped);
}

#[test]
fn gate_is_not_a_basic_land_type() {
    cr!("205.3i", "305.6");
    ruling!("Simic Guildgate", "Gate is not a basic land type.");
    supported("Simic Guildgate");
    supported("Matca Rioters");
    // Matca Rioters: "Domain — ~'s power and toughness are each equal to the number of
    // basic land types among lands you control."
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Simic Guildgate");
    assert_eq!(subtypes_now(&t, gate), vec!["Gate"]);
    let rioters = t.battlefield(P0, "Matca Rioters");
    assert_eq!(t.pt(rioters), (0, 0));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Simic Guildgate");
    t.battlefield(P0, "Forest");
    let rioters = t.battlefield(P0, "Matca Rioters");
    assert_eq!(t.pt(rioters), (1, 1));
}

#[test]
fn changing_a_lands_type_leaves_snow_alone() {
    cr!("305.7", "205.4g");
    ruling!(
        "Phantasmal Terrain",
        "Will not add or remove the supertype snow to or from a land."
    );
    supported("Phantasmal Terrain");
    supported("Conversion");
    // Phantasmal Terrain: "As this Aura enters, choose a basic land type. Enchanted land
    // is the chosen type." Snow-Covered Forest becomes an Island.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Snow-Covered Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.lands(P0, "Island", 2);
    let terrain = t.hand(P0, "Phantasmal Terrain");
    t.cast(P0, terrain).target(land).go();
    t.resolve_all();
    assert_eq!(subtypes_now(&t, land), vec!["Island"]);
    let sup = t.obj(land).chars.supertypes;
    assert!(sup.contains(Supertype::Snow) && sup.contains(Supertype::Basic));
    assert_eq!(mana_from(&mut t, P0, land, "{U}"), vec![ManaType::U]);
    // Conversion: "All Mountains are Plains." A Plains is not made snow, nor a snowy
    // Mountain unsnowed.
    let mut t = TestGame::new(2);
    let snowy = t.battlefield(P0, "Snow-Covered Mountain");
    let plain = t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Conversion");
    t.g.recompute();
    assert_eq!(subtypes_now(&t, snowy), vec!["Plains"]);
    assert!(t.obj(snowy).chars.supertypes.contains(Supertype::Snow));
    assert!(!t.obj(plain).chars.supertypes.contains(Supertype::Snow));
}

#[test]
fn a_lair_is_only_a_lair_and_not_basic() {
    cr!("205.3i", "205.4c");
    ruling!(
        "Crosis's Catacombs",
        "This land is of type \"Lair\" only; other subtypes have been removed. It is not a basic land."
    );
    supported("Crosis's Catacombs");
    // "When this land enters, sacrifice it unless you return a non-Lair land you control
    // to its owner's hand."
    let mut t = TestGame::new(2);
    let cavern = t.battlefield(P0, "Dromar's Cavern");
    let island = t.battlefield(P0, "Island");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    let catacombs = t.enter(P0, "Crosis's Catacombs");
    assert_eq!(subtypes_now(&t, catacombs), vec!["Lair"]);
    assert!(!basic_now(&t, catacombs));
    t.resolve_all();
    let offered = all_candidates(&t, from);
    assert!(!offered.contains(&Entity::Object(cavern)), "{offered:?}");
    assert!(t.in_hand(P0, "Island"), "{:?}", t.zone(island));
    assert!(t.on_battlefield(catacombs));
}

#[test]
fn a_snarl_reveals_any_land_card_with_the_types_but_not_another_snarl() {
    cr!("205.3i", "614.1c");
    ruling!(
        "Frostboil Snarl",
        "You may reveal any land card with either or both of the appropriate subtypes. It doesn't have to be a basic land card."
    );
    ruling!(
        "Frostboil Snarl",
        "The \"Snarl\" itself doesn't have any land subtypes. You can't reveal one to satisfy the ability of another."
    );
    supported("Frostboil Snarl");
    // "As this land enters, you may reveal an Island or Mountain card from your hand. If
    // you don't, this land enters tapped."
    for (in_hand, tapped) in [("Volcanic Island", false), ("Frostboil Snarl", true)] {
        let mut t = TestGame::new(2);
        let c = t.hand(P0, in_hand);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(c)]);
        let snarl = t.enter(P0, "Frostboil Snarl");
        assert_eq!(t.obj(snarl).tapped, tapped, "revealing {in_hand}");
        assert!(subtypes_now(&t, snarl).is_empty());
    }
}

#[test]
fn a_land_put_onto_the_battlefield_tapped_isnt_untapped_by_its_opponents_clause() {
    cr!("614.1c", "614.12");
    ruling!(
        "Rejuvenating Springs",
        "If an effect puts the land onto the battlefield tapped, having two or more opponents won't untap it."
    );
    supported("Rejuvenating Springs");
    supported("Splendid Reclamation");
    // "This land enters tapped unless you have two or more opponents."
    let mut t = TestGame::new(3);
    let springs = t.enter(P0, "Rejuvenating Springs");
    assert!(!t.obj(springs).tapped, "two opponents");
    let mut t = TestGame::new(2);
    let springs = t.enter(P0, "Rejuvenating Springs");
    assert!(t.obj(springs).tapped, "one opponent");
    // Splendid Reclamation: "Return all land cards from your graveyard to the battlefield
    // tapped."
    let mut t = TestGame::new(3);
    t.graveyard(P0, "Rejuvenating Springs");
    cast_new(&mut t, P0, "Splendid Reclamation", &[]);
    t.resolve_all();
    let springs = t.named_on_battlefield("Rejuvenating Springs");
    assert_eq!(springs.len(), 1);
    assert!(t.obj(springs[0]).tapped);
}

#[test]
fn a_castle_sees_a_nonbasic_land_with_a_basic_land_type() {
    cr!("305.8", "614.1c");
    ruling!(
        "Castle Ardenvale",
        "Although the common lands have basic land types, they aren't basic lands."
    );
    supported("Castle Ardenvale");
    // "This land enters tapped unless you control a Plains."
    let mut t = TestGame::new(2);
    let castle = t.enter(P0, "Castle Ardenvale");
    assert!(t.obj(castle).tapped);
    let fountain = t.battlefield(P0, "Hallowed Fountain");
    assert!(!basic_now(&t, fountain));
    let castle = t.enter(P0, "Castle Ardenvale");
    assert!(!t.obj(castle).tapped, "Hallowed Fountain is a Plains");
}

#[test]
fn a_land_creature_is_both_a_land_and_a_creature() {
    cr!("205.2a", "205.1b", "613.1d");
    ruling!(
        "Sylvan Advocate",
        "A “land creature” is a permanent that’s both a land and a creature."
    );
    supported("Sylvan Advocate");
    supported("Kamahl, Fist of Krosa");
    // "As long as you control six or more lands, this creature and land creatures you
    // control get +2/+2." Kamahl: "{G}: Target land becomes a 1/1 creature until end of
    // turn. It's still a land."
    let mut t = TestGame::new(2);
    let advocate = t.battlefield(P0, "Sylvan Advocate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let kamahl = t.battlefield(P0, "Kamahl, Fist of Krosa");
    let forests = t.lands(P0, "Forest", 5);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    t.g.recompute();
    assert_eq!(t.pt(advocate), (4, 5));
    assert_eq!(t.pt(arbor), (3, 3), "Dryad Arbor is a land creature");
    assert_eq!(t.pt(bears), (2, 2), "not a land");
    assert_eq!(t.pt(kamahl), (4, 3), "not a land");
    let target = forests[4];
    add_mana(&mut t, P0, ManaType::G, 1);
    t.activate(P0, kamahl, 0, &[Entity::Object(target)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(target), (3, 3));
}

#[test]
fn a_creature_card_is_any_card_with_the_type_creature() {
    cr!("205.2a", "109.2");
    ruling!(
        "Gravedigger",
        "A \"creature card\" is any card with the type creature, even if it has other types such as artifact, enchantment, or land."
    );
    supported("Gravedigger");
    // "When this creature enters, you may return target creature card from your
    // graveyard to your hand."
    let mut t = TestGame::new(2);
    let arbor = t.graveyard(P0, "Dryad Arbor");
    let thopter = t.graveyard(P0, "Ornithopter");
    let forest = t.graveyard(P0, "Forest");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(arbor)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Gravedigger");
    t.settle();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .flat_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => {
                candidates.clone()
            }
            _ => vec![],
        })
        .collect();
    assert!(offered.contains(&Entity::Object(arbor)), "{offered:?}");
    assert!(offered.contains(&Entity::Object(thopter)), "{offered:?}");
    assert!(!offered.contains(&Entity::Object(forest)), "{offered:?}");
    t.resolve_all();
    assert!(t.in_hand(P0, "Dryad Arbor"));
}

#[test]
fn a_text_change_doesnt_change_card_names() {
    cr!("612.1", "612.2");
    ruling!(
        "Mind Bend",
        "You can’t change proper nouns (i.e. card names) such as “Island Fish Jasconius”."
    );
    supported("Mind Bend");
    supported("Island Fish Jasconius");
    // Mind Bend changes Island to Swamp in Island Fish Jasconius's text ("When you
    // control no Islands, sacrifice this creature."): its name stays.
    let mut t = TestGame::new(2);
    let fish = t.battlefield(P0, "Island Fish Jasconius");
    let island = t.battlefield(P0, "Island");
    t.battlefield(P0, "Swamp");
    let bend = t.hand(P0, "Mind Bend");
    // Options: the five colors, then Plains, Island, Swamp, Mountain, Forest.
    t.answer(P0, DecisionKind::Option, Answer::Index(6));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, bend).target(fish).go();
    t.resolve_all();
    assert_eq!(t.obj(fish).chars.name, "Island Fish Jasconius");
    // Its text now says Swamps: losing the Island doesn't matter.
    destroy(&mut t, island);
    t.resolve_all();
    assert!(t.on_battlefield(fish));
    assert_eq!(t.zone(fish), Zone::Battlefield);
}

#[test]
fn wastes_is_not_a_land_type() {
    cr!("205.3i", "305.6");
    ruling!(
        "Wastes",
        "Wastes is not a land type. If something asks you to name a land type, you can't choose Wastes."
    );
    supported("Wastes");
    let mut t = TestGame::new(2);
    let wastes = t.battlefield(P0, "Wastes");
    assert!(subtypes_now(&t, wastes).is_empty());
    assert!(basic_now(&t, wastes));
    // Phantasmal Terrain's "choose a basic land type": the five basic land types.
    let land = t.battlefield(P0, "Forest");
    t.lands(P0, "Island", 2);
    let terrain = t.hand(P0, "Phantasmal Terrain");
    let from = t.asked().len();
    t.cast(P0, terrain).target(land).go();
    t.resolve_all();
    let offered = options_offered(&t, from);
    assert_eq!(
        offered,
        vec![vec!["Plains", "Island", "Swamp", "Mountain", "Forest"]]
    );
}

#[test]
fn lands_that_become_creatures_keep_their_other_types_and_abilities() {
    cr!("205.1b", "613.1d", "305.6");
    ruling!(
        "Kamahl, Fist of Krosa",
        "Lands that become creatures retain any other supertypes, card types, subtypes, and abilities they have."
    );
    let mut t = TestGame::new(2);
    let kamahl = t.battlefield(P0, "Kamahl, Fist of Krosa");
    let land = t.battlefield(P0, "Snow-Covered Forest");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.activate(P0, kamahl, 0, &[Entity::Object(land)]).unwrap();
    t.resolve_all();
    let o = t.obj(land);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.chars.supertypes.contains(Supertype::Snow));
    assert_eq!(subtypes_now(&t, land), vec!["Forest"]);
    assert_eq!(t.pt(land), (1, 1));
    assert_eq!(mana_from(&mut t, P0, land, "{G}"), vec![ManaType::G]);
}
