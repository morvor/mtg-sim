//! CR 205: the type line — card types, subtypes and their lists, and supertypes.

use crate::r105_util::{last_options, matches, spell_target_candidates};
use crate::r200_common::*;
use crate::r300_common::{can_cast, subtypes_of, types_of};
use crate::r609_common::{continuous, permanent};
use crate::r703_common::{
    add_scheme_deck, archenemy_game, keyword_action, oracle_card, run_effect,
};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::ability::KeywordAction;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The list of names a rule of CR 205.3 gives ("... types are A, B (see rule ...), and
/// C."), read directly from the rule text.
fn cr_list(rule: &str) -> Vec<String> {
    let text = mtg_data::comprehensive_rules().get(rule).unwrap().text.clone();
    let list = text
        .rsplit_once(" types are ")
        .or_else(|| text.rsplit_once(" type is "))
        .map(|(_, l)| l)
        .unwrap();
    // The list ends at the end of its sentence ("... and Urza's. Of that list, ...").
    let list = list.split(". ").next().unwrap();
    list.trim_end_matches('.')
        .split(", ")
        .map(|s| {
            let s = s.strip_prefix("and ").unwrap_or(s);
            s.split(" (").next().unwrap().replace('\u{2019}', "'")
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 205.2: card types
// ---------------------------------------------------------------------------

#[test]
fn the_fifteen_card_types() {
    cr!("205.2a");
    let words = [
        "artifact",
        "battle",
        "conspiracy",
        "creature",
        "dungeon",
        "enchantment",
        "instant",
        "kindred",
        "land",
        "phenomenon",
        "plane",
        "planeswalker",
        "scheme",
        "sorcery",
        "vanguard",
    ];
    assert_eq!(CardType::ALL.len(), words.len());
    for (w, t) in words.iter().zip(CardType::ALL) {
        assert_eq!(CardType::from_word(w), Some(t));
        assert_eq!(t.word(), *w);
    }
    // "Tribal" is now "kindred".
    assert_eq!(CardType::from_word("Tribal"), Some(CardType::Kindred));
    // Real cards of each type.
    for (name, t) in [
        ("Sol Ring", CardType::Artifact),
        ("Invasion of Tarkir", CardType::Battle),
        ("Backup Plan", CardType::Conspiracy),
        ("Grizzly Bears", CardType::Creature),
        ("Tomb of Annihilation", CardType::Dungeon),
        ("Glorious Anthem", CardType::Enchantment),
        ("Lightning Bolt", CardType::Instant),
        ("Bitterblossom", CardType::Kindred),
        ("Forest", CardType::Land),
        ("Chaotic Aether", CardType::Phenomenon),
        ("Academy at Tolaria West", CardType::Plane),
        ("Jace Beleren", CardType::Planeswalker),
        ("A Display of My Dark Power", CardType::Scheme),
        ("Demonic Tutor", CardType::Sorcery),
        ("Titania", CardType::Vanguard),
    ] {
        assert!(types_of(name).contains(t), "{name} is a {t:?}");
    }
    // "Token" and other words on a type line aren't card types.
    assert_eq!(
        TypeLine::parse("Token Creature — Goblin").card_types,
        CardTypeSet::single(CardType::Creature)
    );
}

#[test]
fn an_object_with_several_card_types_matches_each() {
    cr!("205.2b");
    let mut t = TestGame::new(2);
    // Ornithopter is an artifact creature: Shatter ("Destroy target artifact") and Doom
    // Blade ("Destroy target nonblack creature") can each target it.
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 2);
    let shatter = t.hand(P0, "Shatter");
    let blade = t.hand(P0, "Doom Blade");
    let a = spell_target_candidates(&t, P0, shatter, 0);
    let c = spell_target_candidates(&t, P0, blade, 0);
    assert!(a.contains(&Entity::Object(thopter)) && a.contains(&Entity::Object(ring)));
    assert!(!a.contains(&Entity::Object(bears)));
    assert!(c.contains(&Entity::Object(thopter)) && c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(ring)));
    t.cast(P0, shatter).target(thopter).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn tokens_and_copies_have_card_types() {
    cr!("205.2c");
    let mut t = TestGame::new(2);
    // Tokens: Spectral Procession's Spirits are creatures ("Destroy target creature").
    t.lands(P0, "Plains", 3);
    let sp = t.hand(P0, "Spectral Procession");
    t.cast(P0, sp).go();
    t.resolve();
    let spirit = t.named_on_battlefield("Spirit Token")[0];
    assert!(t.obj_now(spirit).is_token());
    assert!(t.obj_now(spirit).chars.is(CardType::Creature));
    t.lands(P1, "Swamp", 2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let blade = t.hand(P1, "Doom Blade");
    assert!(spell_target_candidates(&t, P1, blade, 0).contains(&Entity::Object(spirit)));
    // Copies of spells: a copy of Lightning Bolt is an instant spell.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert!(!t.obj_now(copy).is_card());
    assert!(t.obj_now(copy).chars.is(CardType::Instant));
    let instant_spell = Filter::and(vec![Filter::Spell, Filter::Type(CardType::Instant)]);
    assert!(matches(&t, copy, &instant_spell, P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

// ---------------------------------------------------------------------------
// 205.3: subtypes
// ---------------------------------------------------------------------------

#[test]
fn subtypes_are_listed_after_the_dash() {
    cr!("205.3a", "205.3b");
    // One or more subtypes; each word is a separate subtype.
    assert_eq!(subtypes_of("Mountain"), vec!["Mountain"]);
    assert_eq!(subtypes_of("Llanowar Elves"), vec!["Elf", "Druid"]);
    assert_eq!(subtypes_of("Bonesplitter"), vec!["Equipment"]);
    assert!(subtypes_of("Lightning Bolt").is_empty());
    let tl = TypeLine::parse("Creature — Goblin Wizard");
    assert_eq!(tl.subtypes, vec!["Goblin", "Wizard"]);
    // Creature subtypes may be two words: Time Lord is one creature type.
    assert_eq!(subtypes_of("The Tenth Doctor"), vec!["Time Lord", "Doctor"]);
    assert!(is_creature_type("Time Lord") && !is_creature_type("Time"));
    // All words after a plane's dash are a single subtype.
    assert_eq!(subtypes_of("Furnace Layer"), vec!["New Phyrexia"]);
    let tl = TypeLine::parse("Plane — Serra's Realm");
    assert_eq!(tl.subtypes, vec!["Serra's Realm"]);
}

#[test]
fn each_subtype_is_correlated_to_its_card_type() {
    // CR 205.3c: Dryad Arbor is a "Land Creature — Forest Dryad": Forest is a land type
    // and Dryad a creature type. When it stops being a creature, it keeps Forest but
    // loses Dryad.
    cr!("205.3c");
    ruling!("Dryad Arbor", "Forest is a land type and Dryad is a creature type.");
    assert_eq!(subtype_kind("Forest"), Some(SubtypeKind::Land));
    assert_eq!(subtype_kind("Dryad"), Some(SubtypeKind::Creature));
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    assert!(t.obj_now(arbor).chars.has_subtype("Forest"));
    assert!(t.obj_now(arbor).chars.has_subtype("Dryad"));
    let not_creature = permanent(
        "Uncreator",
        &[CardType::Enchantment],
        vec![continuous(
            Filter::Type(CardType::Land),
            vec![Modification::RemoveTypes(vec![CardType::Creature])],
        )],
    );
    t.custom(P0, not_creature, Zone::Battlefield);
    t.g.recompute();
    let c = &t.obj_now(arbor).chars;
    assert!(!c.is(CardType::Creature));
    assert!(c.has_subtype("Forest"));
    assert!(!c.has_subtype("Dryad"));
}

#[test]
fn choosing_a_subtype_means_exactly_one_existing_subtype_of_the_right_kind() {
    cr!("205.3e");
    ruling!("Xenograft", "You must choose an existing _Magic_ creature type.");
    // Xenograft: "As this enchantment enters, choose a creature type."
    let mut t = TestGame::new(2);
    let x = t.enter(P0, "Xenograft");
    let opts = last_options(&t, P0);
    for ok in ["Merfolk", "Wizard", "Time Lord", "Human"] {
        assert!(opts.iter().any(|o| o == ok), "{ok} is a creature type");
    }
    for bad in ["Merfolk Wizard", "Artifact", "Opponent", "Swamp", "Truck", "Equipment"] {
        assert!(!opts.iter().any(|o| o == bad), "{bad} can't be chosen");
    }
    let chosen = t.obj_now(x).choices.creature_type.clone().unwrap();
    assert!(is_creature_type(&chosen));
    // An answer that isn't one of the options still results in one creature type.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(100_000));
    let x = t.enter(P0, "Xenograft");
    let chosen = t.obj_now(x).choices.creature_type.clone().unwrap();
    assert!(is_creature_type(&chosen));
}

#[test]
fn subtypes_come_from_the_oracle_card_reference() {
    // CR 205.3f: Ornithopter was printed as an "Artifact Creature" with no subtype; it
    // has since received the creature type Thopter.
    cr!("205.3f");
    assert_eq!(subtypes_of("Ornithopter"), vec!["Thopter"]);
    // Grizzly Bears was printed as "Summon Bears"; it's a Creature — Bear.
    assert_eq!(subtypes_of("Grizzly Bears"), vec!["Bear"]);
    assert!(types_of("Grizzly Bears").contains(CardType::Creature));
}

/// Checks that every name on the CR list `rule` is a subtype of `kind`.
fn check_list(rule: &str, kind: SubtypeKind, min: usize) {
    let list = cr_list(rule);
    assert!(list.len() >= min, "{rule}: {list:?}");
    for s in &list {
        assert!(subtype_kinds(s).contains(&kind), "{rule}: {s}");
    }
}

#[test]
fn artifact_types() {
    cr!("205.3g");
    check_list("205.3g", SubtypeKind::Artifact, 20);
    // A Treasure token is an artifact with the artifact type Treasure; Bonesplitter is an
    // Equipment.
    assert!(subtypes_of("Bonesplitter").contains(&"Equipment".to_string()));
    assert!(!is_creature_type("Treasure") && !is_creature_type("Map"));
    // An artifact type is correlated with the artifact card type: an Equipment that stops
    // being an artifact stops being an Equipment (CR 205.1a).
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bonesplitter");
    assert!(t.obj_now(b).chars.has_subtype("Equipment"));
    let remover = permanent(
        "Unartifacter",
        &[CardType::Enchantment],
        vec![continuous(
            Filter::Type(CardType::Artifact),
            vec![
                Modification::AddTypes(vec![CardType::Enchantment]),
                Modification::RemoveTypes(vec![CardType::Artifact]),
            ],
        )],
    );
    t.custom(P0, remover, Zone::Battlefield);
    t.g.recompute();
    assert!(!t.obj_now(b).chars.has_subtype("Equipment"));
    // Spacecraft is both an artifact type and a planar type (CR 205.3n): as an artifact
    // type it stays with an artifact.
    assert!(subtype_kinds("Spacecraft").contains(&SubtypeKind::Artifact));
    assert!(subtype_kinds("Spacecraft").contains(&SubtypeKind::Plane));
}

#[test]
fn enchantment_types() {
    cr!("205.3h");
    check_list("205.3h", SubtypeKind::Enchantment, 12);
    assert_eq!(subtypes_of("Pacifism"), vec!["Aura"]);
    assert!(subtypes_of("Curse of Death's Hold").contains(&"Curse".to_string()));
}

#[test]
fn land_types_and_basic_land_types() {
    cr!("205.3i");
    check_list("205.3i", SubtypeKind::Land, 15);
    for b in ["Forest", "Island", "Mountain", "Plains", "Swamp"] {
        assert!(is_basic_land_type(b));
    }
    for other in ["Cave", "Desert", "Gate", "Urza's", "Power-Plant", "Tower"] {
        assert_eq!(subtype_kind(other), Some(SubtypeKind::Land));
        assert!(!is_basic_land_type(other));
    }
    // "Land — Urza's Tower": two land types.
    assert_eq!(subtypes_of("Urza's Tower"), vec!["Urza's", "Tower"]);
    // Tropical Island has two basic land types (and so their mana abilities, CR 305.6).
    assert_eq!(subtypes_of("Tropical Island"), vec!["Forest", "Island"]);
}

#[test]
fn planeswalker_types() {
    cr!("205.3j");
    check_list("205.3j", SubtypeKind::Planeswalker, 80);
    assert_eq!(subtypes_of("Jace Beleren"), vec!["Jace"]);
    assert_eq!(subtypes_of("Karn, Scion of Urza"), vec!["Karn"]);
}

#[test]
fn spell_types() {
    cr!("205.3k");
    assert_eq!(
        cr_list("205.3k"),
        vec!["Adventure", "Arcane", "Lesson", "Omen", "Trap"]
    );
    check_list("205.3k", SubtypeKind::Spell, 5);
    assert_eq!(subtypes_of("Lava Spike"), vec!["Arcane"]);
    // An instant and a sorcery share the list: Petty Theft is an Instant — Adventure.
    let def = card("Brazen Borrower");
    assert!(def.faces[1].chars.has_subtype("Adventure"));
    assert!(def.faces[1].chars.is(CardType::Instant));
}

#[test]
fn planar_types() {
    cr!("205.3n");
    check_list("205.3n", SubtypeKind::Plane, 60);
    assert_eq!(subtypes_of("Academy at Tolaria West"), vec!["Dominaria"]);
    // Multi-word planar types are one subtype.
    assert_eq!(subtype_kind("New Phyrexia"), Some(SubtypeKind::Plane));
    assert_eq!(subtypes_of("Furnace Layer"), vec!["New Phyrexia"]);
}

#[test]
fn the_dungeon_type_undercity() {
    cr!("205.3p");
    assert_eq!(cr_list("205.3p"), vec!["Undercity"]);
    assert_eq!(subtype_kind("Undercity"), Some(SubtypeKind::Dungeon));
    let undercity = card("Undercity");
    let c = &undercity.front().chars;
    assert!(c.is(CardType::Dungeon));
    assert_eq!(c.subtypes.to_vec(), vec!["Undercity"]);
    // Other dungeons have no subtype.
    assert!(subtypes_of("Tomb of Annihilation").is_empty());
    // The dungeon type is correlated with the dungeon card type: an object that stops
    // being a dungeon loses it (CR 205.1a).
    let mut t = TestGame::new(2);
    let u = t.command(P0, "Undercity");
    let remover = permanent(
        "Undungeoner",
        &[CardType::Enchantment],
        vec![continuous(
            Filter::and(vec![
                Filter::Type(CardType::Dungeon),
                Filter::InZone(ZoneKind::Command),
            ]),
            vec![
                Modification::AddTypes(vec![CardType::Artifact]),
                Modification::RemoveTypes(vec![CardType::Dungeon]),
            ],
        )],
    );
    assert!(t.obj_now(u).chars.has_subtype("Undercity"));
    t.custom(P0, remover, Zone::Battlefield);
    t.g.recompute();
    assert!(!t.obj_now(u).chars.has_subtype("Undercity"));
}

#[test]
fn the_battle_type_siege() {
    cr!("205.3q");
    assert_eq!(cr_list("205.3q"), vec!["Siege"]);
    assert_eq!(subtype_kind("Siege"), Some(SubtypeKind::Battle));
    assert_eq!(subtypes_of("Invasion of Tarkir"), vec!["Siege"]);
}

#[test]
fn some_card_types_have_no_subtypes() {
    cr!("205.3r");
    for name in [
        "Chaotic Aether",
        "A Display of My Dark Power",
        "Titania",
        "Backup Plan",
    ] {
        assert!(subtypes_of(name).is_empty(), "{name}");
    }
    // Their type lines list no subtypes; ongoing schemes have a supertype only.
    let tl = TypeLine::parse("Ongoing Scheme");
    assert!(tl.card_types.contains(CardType::Scheme) && tl.subtypes.is_empty());
    assert!(subtypes_of("Your Inescapable Doom").is_empty());
}

// ---------------------------------------------------------------------------
// 205.4: supertypes
// ---------------------------------------------------------------------------

#[test]
fn supertypes_are_independent_of_card_types_and_subtypes() {
    cr!("205.4b");
    ruling!(
        "Leyline of Singularity",
        "Adding a supertype doesn't overwrite other supertypes."
    );
    let mut t = TestGame::new(2);
    // Nature's Revolt: "All lands are 2/2 creatures that are still lands." A legendary
    // land stays legendary; a snow land stays snow.
    t.battlefield(P0, "Nature's Revolt");
    let pendel = t.battlefield(P0, "Pendelhaven");
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    t.g.recompute();
    let c = &t.obj_now(pendel).chars;
    assert!(c.is(CardType::Creature) && c.is(CardType::Land));
    assert!(c.is_legendary());
    let c = &t.obj_now(snow).chars;
    assert!(c.is(CardType::Creature));
    assert!(c.has_supertype(Supertype::Snow) && c.has_supertype(Supertype::Basic));
    assert!(c.has_subtype("Forest"));
    // Gaining a supertype keeps the others: Leyline of Singularity makes nonland
    // permanents legendary; Adarkar Windform stays snow.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Singularity");
    let windform = t.battlefield(P0, "Adarkar Windform");
    t.g.recompute();
    let c = &t.obj_now(windform).chars;
    assert!(c.is_legendary() && c.has_supertype(Supertype::Snow));
    assert!(c.is(CardType::Creature) && c.has_subtype("Illusion"));
}

#[test]
fn basic_lands_are_lands_with_the_basic_supertype() {
    cr!("205.4c");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let snow = t.battlefield(P1, "Snow-Covered Forest");
    let trop = t.battlefield(P1, "Tropical Island");
    let basic = Filter::and(vec![
        Filter::Supertype(Supertype::Basic),
        Filter::Type(CardType::Land),
    ]);
    assert!(matches(&t, forest, &basic, P0));
    assert!(matches(&t, snow, &basic, P0));
    // Tropical Island has basic land types but isn't basic.
    assert!(!matches(&t, trop, &basic, P0));
    assert!(t.obj_now(trop).chars.has_subtype("Forest"));
    // Wasteland: "Destroy target nonbasic land."
    let waste = t.battlefield(P0, "Wasteland");
    let targets: Vec<Entity> = {
        let ab = t
            .obj_now(waste)
            .chars
            .abilities
            .iter()
            .find_map(|a| match &a.kind {
                AbilityKind::Activated(x) if !x.is_mana_ability => Some(x.body.targets[0].clone()),
                _ => None,
            })
            .unwrap();
        t.g.legal_target_candidates(&ab, &mtg_engine::eval::Ctx::new(Some(waste), P0), waste)
    };
    assert!(targets.contains(&Entity::Object(trop)));
    assert!(!targets.contains(&Entity::Object(forest)));
    assert!(!targets.contains(&Entity::Object(snow)));
}

#[test]
fn legendary_permanents_are_subject_to_the_legend_rule() {
    cr!("205.4d");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    let b = t.battlefield(P0, "Isamaru, Hound of Konda");
    // Nonlegendary permanents with the same name aren't.
    let c = t.battlefield(P0, "Grizzly Bears");
    let d = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!([a, b].iter().filter(|x| t.g.is_live(**x)).count(), 1);
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
    assert!(t.g.is_live(c) && t.g.is_live(d));
}

#[test]
fn legendary_sorceries_need_a_legendary_creature_or_planeswalker() {
    cr!("205.4e");
    ruling!(
        "Jaya's Immolating Inferno",
        "You can't cast a legendary sorcery unless you control a legendary creature or a legendary planeswalker."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let inferno = t.hand(P0, "Jaya's Immolating Inferno");
    t.battlefield(P0, "Grizzly Bears");
    // A legendary noncreature artifact isn't enough.
    t.battlefield(P0, "Mox Opal");
    assert!(!can_cast(&mut t, P0, inferno));
    assert!(t.cast(P0, inferno).x(1).target(P1).try_go().is_err());
    assert_eq!(t.zone(inferno), Zone::Hand(P0));
    t.battlefield(P0, "Isamaru, Hound of Konda");
    assert!(can_cast(&mut t, P0, inferno));
    t.cast(P0, inferno).x(1).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A legendary planeswalker also allows it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let inferno = t.hand(P0, "Jaya's Immolating Inferno");
    assert!(!can_cast(&mut t, P0, inferno));
    t.battlefield(P0, "Jace Beleren");
    assert!(can_cast(&mut t, P0, inferno));
}

#[test]
fn world_permanents_are_subject_to_the_world_rule() {
    cr!("205.4f");
    ruling!(
        "Concordant Crossroads",
        "When a world permanent enters, any world permanents that were already on the battlefield are put into their owners' graveyards."
    );
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Concordant Crossroads");
    t.settle();
    let second = t.battlefield(P1, "Concordant Crossroads");
    t.settle();
    // Only the one that has had the world supertype for the shortest time remains.
    assert!(!t.g.is_live(first));
    assert!(t.g.is_live(second));
}

#[test]
fn snow_permanents_have_the_snow_supertype() {
    cr!("205.4g");
    ruling!("Snow-Covered Forest", "Snow is a supertype, not a card type.");
    ruling!(
        "Snow-Covered Forest",
        "It represents a cost that can be paid by one mana that was produced by a snow source."
    );
    let mut t = TestGame::new(2);
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    let forest = t.battlefield(P0, "Forest");
    // Snow Fortress isn't snow despite its name.
    let fortress = t.battlefield(P0, "Snow Fortress");
    let f = Filter::Supertype(Supertype::Snow);
    assert!(matches(&t, snow, &f, P0));
    assert!(!matches(&t, forest, &f, P0));
    assert!(!matches(&t, fortress, &f, P0));
    // {S} can be paid only with mana from a snow source: Icehide Troll's "{S}{S}: ...".
    let troll = t.battlefield(P0, "Icehide Troll");
    t.g.objects[snow.0 as usize].tapped = true;
    t.lands(P0, "Forest", 1);
    assert!(t.activate(P0, troll, 0, &[]).is_err());
    t.g.objects[snow.0 as usize].tapped = false;
    t.battlefield(P0, "Snow-Covered Forest");
    assert!(t.activate(P0, troll, 0, &[]).is_ok());
}

#[test]
fn ongoing_schemes_are_exempt_from_the_scheme_rule() {
    cr!("205.4h");
    let mut t = archenemy_game();
    let ongoing = oracle_card("Standing Plot", "Ongoing Scheme", "", None, "");
    let plain = oracle_card("Passing Plot", "Scheme", "", None, "");
    assert!(TypeLine::parse("Ongoing Scheme")
        .supertypes
        .contains(Supertype::Ongoing));
    let deck = add_scheme_deck(&mut t, P0, vec![ongoing, plain]);
    keyword_action(&mut t, P0, KeywordAction::SetInMotion, 2);
    t.settle();
    // The non-ongoing scheme is turned face down and put on the bottom; the ongoing one
    // stays face up.
    assert!(!t.obj_now(deck[0]).face_down);
    assert!(t.obj_now(deck[1]).face_down);
    assert_eq!(variants::face_up_schemes(&t.g), vec![t.g.current(deck[0])]);
}
