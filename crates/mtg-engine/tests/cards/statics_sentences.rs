//! Static abilities spanning several sentences ("It's still a land.", "Otherwise, ...",
//! "It can't attack and loses all abilities."), supertype changes, "loses all other
//! abilities", "can't have or gain", relative clauses in subjects, and irregular plurals.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn enchant(t: &mut TestGame, p: PlayerId, aura: &str, target: ObjectId) -> ObjectId {
    let a = t.battlefield(p, aura);
    t.attach(a, Entity::Object(target));
    t.settle();
    a
}

// ---------------------------------------------------------------------------
// "It's still a land."
// ---------------------------------------------------------------------------

#[test]
fn zendikon_land_is_a_creature_that_is_still_a_land() {
    cr!("205.1b", "613.1d", "613.4b");
    ruling!(
        "Wind Zendikon",
        "The enchanted permanent will be both a land and a creature"
    );
    compiles("Wind Zendikon");
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Island");
    enchant(&mut t, P0, "Wind Zendikon", island);
    let o = t.obj_now(island);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.subtypes.iter().any(|s| s == "Island"));
    assert!(o.chars.subtypes.iter().any(|s| s == "Elemental"));
    assert!(o.chars.colors.contains(Color::Blue));
    assert_eq!(t.pt(island), (2, 2));
    assert!(has(&t, island, KeywordKind::Flying));
}

#[test]
fn enchanted_mountain_keeps_its_land_types_and_mana_ability() {
    cr!("205.1b");
    ruling!(
        "Awaken the Ancient",
        "The enchanted Mountain retains any other types or subtypes it may have."
    );
    compiles("Awaken the Ancient");
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    enchant(&mut t, P0, "Awaken the Ancient", mountain);
    assert_eq!(t.pt(mountain), (7, 7));
    assert!(has(&t, mountain, KeywordKind::Haste));
    let o = t.obj_now(mountain);
    assert!(o.chars.subtypes.iter().any(|s| s == "Mountain"));
    assert!(o.chars.subtypes.iter().any(|s| s == "Giant"));
    assert!(o.is(CardType::Land));
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, mountain, 0, &[]).is_ok());
}

#[test]
fn lands_you_control_are_creatures_that_are_still_lands() {
    cr!("205.1b", "613.4b");
    compiles("Ambush Commander");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ambush Commander");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Forest");
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.subtypes.iter().any(|s| s == "Elf"));
    assert_eq!(t.pt(forest), (1, 1));
    assert!(!t.obj_now(theirs).is(CardType::Creature));
}

// ---------------------------------------------------------------------------
// "Otherwise, ..."
// ---------------------------------------------------------------------------

#[test]
fn otherwise_applies_when_the_condition_is_false() {
    cr!("611.3a", "613.4c");
    compiles("Gift of Fangs");
    let mut t = TestGame::new(2);
    let vampire = t.battlefield(P0, "Vampire Interloper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "Gift of Fangs", vampire);
    enchant(&mut t, P1, "Gift of Fangs", bears);
    // "+2/+2 as long as it's a Vampire. Otherwise, it gets -2/-2."
    assert_eq!(t.pt(vampire), (4, 3));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn otherwise_restriction_after_a_bonus() {
    cr!("611.3a", "508.1c");
    compiles("Bonds of Faith");
    let mut t = TestGame::new(2);
    let human = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "Bonds of Faith", human);
    enchant(&mut t, P0, "Bonds of Faith", bears);
    assert_eq!(t.pt(human), (4, 3));
    assert_eq!(t.pt(bears), (2, 2));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.can_attack(human));
    assert!(!t.can_attack(bears));
}

#[test]
fn as_long_as_you_control_enchanted_creature_otherwise() {
    cr!("611.3a", "509.1b");
    compiles("Mishra's Domination");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    enchant(&mut t, P0, "Mishra's Domination", mine);
    enchant(&mut t, P0, "Mishra's Domination", theirs);
    assert_eq!(t.pt(mine), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
    // P0 doesn't control the other one: it can't block.
    assert!(t.can_block_at_all(mine));
    assert!(!t.can_block_at_all(theirs));
}

// ---------------------------------------------------------------------------
// Several sentences, "loses all other abilities"
// ---------------------------------------------------------------------------

#[test]
fn loses_all_other_abilities_keeps_the_granted_one() {
    cr!("613.1f", "613.4b");
    ruling!(
        "Stasis Field",
        "If the enchanted creature gains an ability after Stasis Field resolves, it will keep that ability."
    );
    compiles("Stasis Field");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    enchant(&mut t, P0, "Stasis Field", angel);
    assert_eq!(t.pt(angel), (0, 2));
    assert!(has(&t, angel, KeywordKind::Defender));
    assert!(!has(&t, angel, KeywordKind::Flying));
    assert!(!has(&t, angel, KeywordKind::Vigilance));
    // A later grant still applies.
    let aura = t.battlefield(P1, "Arcane Flight");
    t.attach(aura, Entity::Object(angel));
    t.settle();
    assert!(has(&t, angel, KeywordKind::Flying));
}

#[test]
fn second_sentence_about_the_same_object() {
    cr!("613.1f", "508.1c");
    compiles("Retro-Mutation");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    enchant(&mut t, P0, "Retro-Mutation", angel);
    let o = t.obj_now(angel);
    assert!(o.chars.subtypes.iter().any(|s| s == "Turtle"));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Angel"));
    assert_eq!(t.pt(angel), (0, 1));
    assert!(!has(&t, angel, KeywordKind::Flying));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!t.can_attack(angel));
}

#[test]
fn black_zombie_in_addition_to_its_other_colors_and_types() {
    cr!("205.1b", "613.1e");
    ruling!("Ghoulflesh", "in addition to its other colors and types");
    compiles("Ghoulflesh");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    enchant(&mut t, P0, "Ghoulflesh", giant);
    let o = t.obj_now(giant);
    assert!(o.chars.colors.contains(Color::Black));
    assert!(o.chars.colors.contains(Color::Red));
    assert!(o.chars.subtypes.iter().any(|s| s == "Zombie"));
    assert!(o.chars.subtypes.iter().any(|s| s == "Giant"));
    assert_eq!(t.pt(giant), (2, 2));
}

// ---------------------------------------------------------------------------
// Supertypes
// ---------------------------------------------------------------------------

#[test]
fn enchanted_permanent_is_legendary() {
    cr!("205.4a", "704.5j");
    ruling!(
        "In Bolas's Clutches",
        "If you control two or more permanents with the same name but only one is legendary"
    );
    compiles("In Bolas's Clutches");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "In Bolas's Clutches", a);
    assert_eq!(t.obj_now(a).controller, P0);
    assert!(t.obj_now(a).chars.supertypes.contains(Supertype::Legendary));
    // Only one of the two is legendary: both stay.
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn enchanted_land_is_snow() {
    cr!("205.4a");
    compiles("Glittering Frost");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    enchant(&mut t, P0, "Glittering Frost", forest);
    assert!(t.obj_now(forest).chars.supertypes.contains(Supertype::Snow));
}

// ---------------------------------------------------------------------------
// "can't have or gain"
// ---------------------------------------------------------------------------

#[test]
fn archetype_removes_and_blocks_the_keyword() {
    cr!("613.1f");
    ruling!(
        "Archetype of Imagination",
        "If you and an opponent each control the same Archetype"
    );
    compiles("Archetype of Imagination");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archetype of Imagination");
    let mine = t.battlefield(P0, "Serra Angel");
    let theirs = t.battlefield(P1, "Serra Angel");
    assert!(has(&t, mine, KeywordKind::Flying));
    assert!(!has(&t, theirs, KeywordKind::Flying));
    // A later grant doesn't give it flying either.
    let aura = t.battlefield(P1, "Arcane Flight");
    t.attach(aura, Entity::Object(theirs));
    t.settle();
    assert!(!has(&t, theirs, KeywordKind::Flying));
    // With one on each side, no creature has flying.
    t.battlefield(P1, "Archetype of Imagination");
    t.settle();
    assert!(!has(&t, mine, KeywordKind::Flying));
}

// ---------------------------------------------------------------------------
// Subjects: relative clauses, irregular plurals
// ---------------------------------------------------------------------------

#[test]
fn each_other_creature_thats_a_fungus_or_saproling() {
    cr!("613.4c");
    ruling!(
        "Sporecrown Thallid",
        "If a creature is somehow both a Fungus and a Saproling, Sporecrown Thallid's ability gives it only +1/+1."
    );
    compiles("Sporecrown Thallid");
    let mut t = TestGame::new(2);
    let thallid = t.battlefield(P0, "Sporecrown Thallid");
    let fungus = t.battlefield(P0, "Thallid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let both = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[both.0 as usize]
        .base
        .subtypes
        .extend(["Fungus".into(), "Saproling".into()]);
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(thallid), (2, 2));
    assert_eq!(t.pt(fungus), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
    // A Fungus Saproling gets +1/+1 once.
    assert_eq!(t.pt(both), (3, 3));
}

#[test]
fn creatures_that_are_zombies_and_or_tokens() {
    cr!("613.4c");
    compiles("On Wings of Gold");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "On Wings of Gold");
    let zombie = t.battlefield(P0, "Walking Corpse");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(zombie), (3, 3));
    assert!(has(&t, zombie, KeywordKind::Flying));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn irregular_plural_subtypes() {
    cr!("205.3m");
    compiles("Crested Sunmare");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crested Sunmare");
    let horse = t.battlefield(P0, "Phyrexian Warhorse");
    assert!(has(&t, horse, KeywordKind::Indestructible));
}

// ---------------------------------------------------------------------------
// "instead as long as", later sentences with their own condition, attachments
// ---------------------------------------------------------------------------

#[test]
fn it_gets_more_instead_as_long_as() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Mind Carver",
        "won't provide additional benefits if more than one opponent has eight or more cards in their graveyard"
    );
    compiles("Mind Carver");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let carver = t.battlefield(P0, "Mind Carver");
    t.attach(carver, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (3, 2));
    for p in [P1, P2] {
        for _ in 0..8 {
            t.graveyard(p, "Island");
        }
    }
    t.settle();
    // +3/+1 instead of +1/+0, not in addition, and only once.
    assert_eq!(t.pt(bears), (5, 3));
}

#[test]
fn later_sentences_with_their_own_conditions() {
    cr!("611.3a", "613.4c");
    compiles("Tenza, Godo's Maul");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let tenza = t.battlefield(P0, "Tenza, Godo's Maul");
    t.attach(tenza, Entity::Object(giant));
    t.settle();
    // Red but not legendary.
    assert_eq!(t.pt(giant), (4, 4));
    assert!(has(&t, giant, KeywordKind::Trample));
    let legend = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.attach(tenza, Entity::Object(legend));
    t.settle();
    assert_eq!(t.pt(legend), (5, 5));
    assert!(!has(&t, legend, KeywordKind::Trample));
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn for_each_aura_attached_to_the_enchanted_creature() {
    cr!("613.4c", "303.4");
    ruling!(
        "Auramancer's Guise",
        "counts itself when determining the power and toughness bonus"
    );
    compiles("Auramancer's Guise");
    compiles("Golem-Skin Gauntlets");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, P0, "Auramancer's Guise", bears);
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has(&t, bears, KeywordKind::Vigilance));
    enchant(&mut t, P0, "Rancor", bears);
    // Rancor: +2/+0; the Guise now counts two Auras.
    assert_eq!(t.pt(bears), (8, 6));
    // Equipment on another creature doesn't count.
    let giant = t.battlefield(P0, "Hill Giant");
    let g = t.battlefield(P0, "Golem-Skin Gauntlets");
    t.attach(g, Entity::Object(giant));
    t.settle();
    assert_eq!(t.pt(giant), (4, 3));
    assert_eq!(t.pt(bears), (8, 6));
}
