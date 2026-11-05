//! Rulings batch P025 — a copy copies exactly what was printed on the original (its
//! copiable values, plus copy effects and the copy's own exceptions) and nothing else:
//! not whether it's tapped, its counters, what's attached to it, or the non-copy effects
//! that changed its power, toughness, types or color (CR 707.2, 707.9). The copied
//! creature is a dressed-up Hill Giant (see [`dressed_giant`]: a red 3/3 that's tapped,
//! equipped, has two +1/+1 counters, +3/+3 and is green-blue too).

use crate::r_p025_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s `copier` enters as a copy of a dressed-up Hill Giant controlled by `giant_p`,
/// which a type-changing effect has also made an Elf artifact.
fn enters_copying_giant(copier: &str, p: PlayerId, giant_p: PlayerId) -> (TestGame, ObjectId) {
    supported(copier);
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, giant_p);
    modify(
        &mut t,
        giant,
        vec![
            Modification::AddTypes(vec![CardType::Artifact]),
            Modification::AddSubtypes(vec!["Elf".into()]),
        ],
    );
    assert!(t.obj_now(giant).is(CardType::Artifact) && has_subtype(&t, giant, "Elf"));
    let c = enter_copying(&mut t, p, copier, giant);
    assert!(!t.obj_now(c).is_token(), "{copier}");
    (t, c)
}

/// One "enters as a copy" case: the copier, who controls the Giant, the copy's power and
/// toughness and a check of its exceptions.
type EnterCase = (
    &'static str,
    PlayerId,
    (i32, i32),
    fn(&TestGame, ObjectId) -> bool,
);

#[test]
fn enter_as_a_copy_copies_only_the_printed_creature() {
    cr!("707.2", "707.9a", "707.9b", "614.1c", "613.1d");
    ruling!(
        "Mercurial Pretender",
        "Mercurial Pretender copies exactly what was printed on the original creature and nothing more (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Mirror Image",
        "Mirror Image copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "Mirrorhall Mimic copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below), except that it's also a Spirit."
    );
    ruling!(
        "Phantasmal Image",
        "Phantasmal Image copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below) and it gains the Illusion creature type and the triggered ability."
    );
    ruling!(
        "Pirated Copy",
        "Pirated Copy copies exactly what was printed on the original creature and nothing more (unless that creature is copying something else or is a token; see below), except it will also be a Pirate and have the granted ability."
    );
    ruling!(
        "Progenitor Mimic",
        "Progenitor Mimic copies exactly what was printed on the original creature and nothing more (unless that creature is copying something else or is a token)."
    );
    ruling!(
        "Sakashima's Student",
        "Sakashima's Student copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below) and it is a Ninja in addition to its other types."
    );
    ruling!(
        "Spark Double",
        "Spark Double copies exactly what was printed on the original permanent (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Stunt Double",
        "Stunt Double copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below)."
    );
    let cases: &[EnterCase] = &[
        ("Mercurial Pretender", P0, (3, 3), |t, c| {
            abilities_with(t, c, "Return") == 1
        }),
        ("Mirror Image", P0, (3, 3), |_, _| true),
        ("Mirrorhall Mimic // Ghastly Mimicry", P1, (3, 3), |t, c| {
            has_subtype(t, c, "Spirit")
        }),
        ("Phantasmal Image", P1, (3, 3), |t, c| {
            has_subtype(t, c, "Illusion") && abilities_with(t, c, "sacrifice it") == 1
        }),
        ("Pirated Copy", P1, (3, 3), |t, c| {
            has_subtype(t, c, "Pirate") && abilities_with(t, c, "you draw a card") == 1
        }),
        ("Progenitor Mimic", P1, (3, 3), |t, c| {
            abilities_with(t, c, "isn't a token") == 1
        }),
        ("Sakashima's Student", P1, (3, 3), |t, c| {
            has_subtype(t, c, "Ninja")
        }),
        // Its exception: one +1/+1 counter of its own (the Giant's two aren't copied).
        ("Spark Double", P0, (4, 4), |t, c| {
            t.counters(c, counters::PLUS1) == 1
        }),
        ("Stunt Double", P1, (3, 3), |_, _| true),
    ];
    for (copier, giant_p, pt, check) in cases {
        let (t, c) = enters_copying_giant(copier, P0, *giant_p);
        assert_eq!(name_of(&t, c), "Hill Giant", "{copier}");
        assert_eq!(t.obj_now(c).chars.colors, red(), "{copier}: colors");
        assert!(has_subtype(&t, c, "Giant"), "{copier}");
        assert!(!has_subtype(&t, c, "Elf"), "{copier}: types");
        assert!(!t.obj_now(c).is(CardType::Artifact), "{copier}: types");
        assert_eq!(t.pt(c), *pt, "{copier}: power and toughness");
        let o = t.obj_now(c);
        assert!(!o.tapped, "{copier}: tapped");
        assert!(
            t.g.attachments_of(Entity::Object(o.id)).is_empty(),
            "{copier}: attachments"
        );
        assert!(check(&t, c), "{copier}: exceptions");
    }
}

#[test]
fn jwari_shapeshifter_copies_only_the_printed_ally() {
    cr!("707.2", "614.1c");
    ruling!(
        "Jwari Shapeshifter",
        "Jwari Shapeshifter copies exactly what was printed on the original creature and nothing more (unless that creature is copying something else or is a token; see below)."
    );
    supported("Jwari Shapeshifter");
    let mut t = TestGame::new(2);
    // Kazandu Blademaster: a white 1/1 Human Soldier Ally with first strike and vigilance.
    let ally = t.battlefield(P1, "Kazandu Blademaster");
    dress(&mut t, P1, ally);
    let c = enter_copying(&mut t, P0, "Jwari Shapeshifter", ally);
    assert_printed(
        &t,
        c,
        "Kazandu Blademaster",
        ColorSet::single(Color::White),
        Some((1, 1)),
        true,
    );
    assert!(has_subtype(&t, c, "Ally"));
}

#[test]
fn mercurial_pretender_copying_an_animated_noncreature_isnt_a_creature() {
    cr!("707.2", "613.1a", "613.1d");
    ruling!(
        "Mercurial Pretender",
        "For example, if Mercurial Pretender enters the battlefield as a copy of a Profane Memento with Ensoul Artifact attached to it, Mercurial Pretender will be a noncreature Profane Memento with Mercurial Pretender’s added activated ability."
    );
    supported("Mercurial Pretender");
    let mut t = TestGame::new(2);
    let memento = t.battlefield(P0, "Profane Memento");
    crate::r_s06_common::attach_new(&mut t, P0, "Ensoul Artifact", memento);
    assert!(is_creature(&t, memento));
    assert_eq!(t.pt(memento), (5, 5));
    let c = enter_copying(&mut t, P0, "Mercurial Pretender", memento);
    assert_eq!(name_of(&t, c), "Profane Memento");
    assert!(t.obj_now(c).is(CardType::Artifact) && !is_creature(&t, c));
    assert_eq!(abilities_with(&t, c, "Return"), 1);
}

#[test]
fn mirrorhall_mimic_copying_an_animated_land_isnt_a_creature() {
    cr!("707.2", "613.1a", "613.1d");
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "Most notably, if it copies a creature that's not normally a creature, it won't be a creature."
    );
    supported("Mirrorhall Mimic // Ghastly Mimicry");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    animate_land(&mut t, land);
    let c = enter_copying(&mut t, P0, "Mirrorhall Mimic // Ghastly Mimicry", land);
    assert_eq!(name_of(&t, c), "Forest");
    assert!(t.obj_now(c).is(CardType::Land) && !is_creature(&t, c));
}

#[test]
fn mindlink_mech_copies_only_the_printed_creature_that_crewed_it() {
    cr!("707.2", "707.9b", "702.122a");
    ruling!(
        "Mindlink Mech",
        "Mindlink Mech copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below), except that it also has Mindlink Mech's types and is 4/3."
    );
    supported("Mindlink Mech");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mindlink Mech");
    let giant = dressed_giant(&mut t, P0);
    untap(&mut t, giant);
    t.answer_targets(P0, &[obj(giant)]);
    assert!(crate::r_s04_common::crew(&mut t, P0, mech, &[giant]));
    t.resolve_all();
    assert_eq!(name_of(&t, mech), "Hill Giant");
    assert_eq!(t.obj_now(mech).chars.colors, red());
    assert!(has_subtype(&t, mech, "Giant") && has_subtype(&t, mech, "Vehicle"));
    assert!(t.obj_now(mech).is(CardType::Artifact) && is_creature(&t, mech));
    assert!(has_kw(&t, mech, KeywordKind::Flying));
    assert_eq!(t.pt(mech), (4, 3));
    assert_eq!(t.counters(mech, counters::PLUS1), 0);
}

// --- Becoming a copy ---------------------------------------------------------------------

#[test]
fn irma_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Irma, Part-Time Mutant",
        "Irma copies exactly what was printed on the original creature and nothing else, with the listed exceptions (unless that creature is copying something else; see below)."
    );
    supported("Irma, Part-Time Mutant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let irma = t.battlefield(P0, "Irma, Part-Time Mutant");
    let giant = dressed_giant(&mut t, P0);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(irma);
    assert_eq!(o.chars.name, "Irma, Part-Time Mutant");
    assert_eq!(o.chars.colors, red());
    assert!(has_subtype(&t, irma, "Giant") && !o.tapped);
    // A printed 3/3 with her own +1/+1 counter only.
    assert_eq!(t.counters(irma, counters::PLUS1), 1);
    assert_eq!(t.pt(irma), (4, 4));
    assert!(t.g.attachments_of(Entity::Object(o.id)).is_empty());
}

#[test]
fn kimahri_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Kimahri, Valiant Guardian",
        "Kimahri copies exactly what was printed on the original creature, with the listed exceptions (unless that creature is copying something else or is a token; see below)."
    );
    supported("Kimahri, Valiant Guardian");
    let mut t = TestGame::new(2);
    let kimahri = t.battlefield(P0, "Kimahri, Valiant Guardian");
    let giant = dressed_giant(&mut t, P1);
    untap(&mut t, giant);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    let o = t.obj_now(kimahri);
    assert_eq!(o.chars.name, "Kimahri, Valiant Guardian");
    assert_eq!(o.chars.colors, red());
    assert!(has_subtype(&t, kimahri, "Giant") && !o.tapped);
    assert!(o.has_keyword(KeywordKind::Vigilance));
    assert_eq!(t.counters(kimahri, counters::PLUS1), 1);
    assert_eq!(t.pt(kimahri), (4, 4));
}

#[test]
fn sarkhan_copies_only_the_printed_dragon() {
    cr!("707.2", "707.9b", "603.3");
    ruling!(
        "Sarkhan, Soul Aflame",
        "Sarkhan copies exactly what was printed on the original Dragon and nothing else, with the noted exceptions (unless that creature is a token; see below)."
    );
    supported("Sarkhan, Soul Aflame");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Soul Aflame");
    t.answer_yes(P0, true);
    let dragon = t.enter(P0, "Shivan Dragon");
    t.settle();
    // In response to Sarkhan's ability, the Dragon is tapped, equipped, gets counters,
    // +3/+3 and becomes green-blue.
    dress(&mut t, P0, dragon);
    assert_eq!(triggers_on_stack(&t).len(), 1);
    t.resolve_all();
    let o = t.obj_now(sarkhan);
    assert_eq!(o.chars.name, "Sarkhan, Soul Aflame");
    assert_eq!(o.chars.colors, red());
    assert!(has_subtype(&t, sarkhan, "Dragon") && legendary(&t, sarkhan));
    assert!(!o.tapped && o.counters.values().all(|n| *n == 0));
    assert_eq!(t.pt(sarkhan), (5, 5));
}

#[test]
fn silent_hallcreeper_copies_only_the_printed_creature() {
    cr!("707.2");
    ruling!(
        "Silent Hallcreeper",
        "Silent Hallcreeper copies exactly what was printed on the target creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    supported("Silent Hallcreeper");
    let mut t = TestGame::new(2);
    let creeper = t.battlefield(P0, "Silent Hallcreeper");
    let giant = dressed_giant(&mut t, P0);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[obj(giant)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(creeper, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Tapped from attacking, but none of the Giant's other changes.
    assert_printed_giant(&t, creeper, Some((3, 3)), false);
}

#[test]
fn sunfrill_imitator_copies_only_the_printed_dinosaur() {
    cr!("707.2", "707.9b");
    ruling!(
        "Sunfrill Imitator",
        "Sunfrill Imitator copies exactly what was printed on the original creature, with the listed exceptions (unless that creature is copying something else or is a token; see below)."
    );
    supported("Sunfrill Imitator");
    let mut t = TestGame::new(2);
    let imitator = t.battlefield(P0, "Sunfrill Imitator");
    // Colossal Dreadmaw: a green 6/6 Dinosaur with trample.
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    dress(&mut t, P0, dino);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(dino)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(imitator, Entity::Player(P1))], &[]);
    t.resolve_all();
    let o = t.obj_now(imitator);
    assert_eq!(o.chars.name, "Sunfrill Imitator");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert!(o.has_keyword(KeywordKind::Trample));
    assert!(o.counters.values().all(|n| *n == 0));
    assert_eq!(t.pt(imitator), (6, 6));
    assert_eq!(t.life(P1), 14);
}

/// `name`'s activated or loyalty ability makes `copier` a copy of `what`: shared shape of
/// the "copies the printed values ... plus any copy effects" rulings.
fn assert_copied_printed_values_plus_copy_effects(t: &TestGame, copier: ObjectId) {
    // A Clone copying a Hill Giant, dressed up: the copy is a red 3/3 Hill Giant.
    assert_printed_giant(t, copier, Some((3, 3)), false);
}

/// P1's Clone copying a Hill Giant, dressed up (tapped, equipped, counters, +3/+3,
/// green-blue).
fn dressed_clone(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let giant = t.battlefield(p, "Hill Giant");
    let clone = clone_of(t, p, giant);
    dress(t, p, clone);
    clone
}

#[test]
fn mirage_mirror_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3", "613.1a");
    ruling!(
        "Mirage Mirror",
        "Mirage Mirror copies the printed values of the target permanent, plus any copy effects that have been applied to it."
    );
    supported("Mirage Mirror");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Mirage Mirror");
    let clone = dressed_clone(&mut t, P1);
    lands_for(&mut t, P0, "{2}");
    t.activate(P0, mirror, 0, &[obj(clone)]).expect("activate");
    t.resolve_all();
    assert_copied_printed_values_plus_copy_effects(&t, mirror);
    // Another Mirror copying an animated land is a land, not a creature.
    let mirror = t.battlefield(P0, "Mirage Mirror");
    let land = t.battlefield(P1, "Forest");
    animate_land(&mut t, land);
    lands_for(&mut t, P0, "{2}");
    t.activate(P0, mirror, 0, &[obj(land)]).expect("activate");
    t.resolve_all();
    assert_eq!(name_of(&t, mirror), "Forest");
    assert!(!is_creature(&t, mirror) && t.obj_now(mirror).is(CardType::Land));
}

#[test]
fn mirror_of_the_forebears_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3", "613.1a");
    ruling!(
        "Mirror of the Forebears",
        "Mirror of the Forebears copies the printed values of the target creature, plus any copy effects that have been applied to it."
    );
    supported("Mirror of the Forebears");
    let mut t = TestGame::new(2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Giant");
    let mirror = t.enter(P0, "Mirror of the Forebears");
    t.settle();
    let clone = dressed_clone(&mut t, P0);
    lands_for(&mut t, P0, "{1}");
    t.activate(P0, mirror, 0, &[obj(clone)]).expect("activate");
    t.resolve_all();
    assert_copied_printed_values_plus_copy_effects(&t, mirror);
    assert!(t.obj_now(mirror).is(CardType::Artifact));
}

#[test]
fn protean_thaumaturge_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3", "613.1a");
    ruling!(
        "Protean Thaumaturge",
        "Protean Thaumaturge copies the printed values of the target creature, plus any copy effects that have been applied to it."
    );
    supported("Protean Thaumaturge");
    let mut t = TestGame::new(2);
    let thaum = t.battlefield(P0, "Protean Thaumaturge");
    let clone = dressed_clone(&mut t, P1);
    // Constellation: an enchantment enters.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(clone)]);
    t.enter(P0, "Leyline of Anticipation");
    t.settle();
    t.resolve_all();
    assert_copied_printed_values_plus_copy_effects(&t, thaum);
    assert_eq!(
        abilities_with(&t, thaum, "Whenever an enchantment you control enters"),
        1
    );
    // An animated land: it becomes a land, not a creature.
    let land = t.battlefield(P1, "Forest");
    animate_land(&mut t, land);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(land)]);
    t.enter(P0, "Leyline of Anticipation");
    t.settle();
    t.resolve_all();
    assert_eq!(name_of(&t, thaum), "Forest");
    assert!(!is_creature(&t, thaum));
}

#[test]
fn saheeli_sublime_artificer_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3", "613.1a", "606.3");
    ruling!(
        "Saheeli, Sublime Artificer",
        "Saheeli's loyalty ability causes the target artifact to copy the printed values of the target permanent, plus any copy effects that have been applied to it."
    );
    supported("Saheeli, Sublime Artificer");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli, Sublime Artificer");
    let servo = t.battlefield(P0, "Ornithopter");
    let clone = dressed_clone(&mut t, P0);
    t.answer_targets(P0, &[obj(servo)]);
    t.answer_targets(P0, &[obj(clone)]);
    activate_containing(&mut t, P0, saheeli, "becomes a copy").expect("activate");
    t.resolve_all();
    assert_copied_printed_values_plus_copy_effects(&t, servo);
    assert!(t.obj_now(servo).is(CardType::Artifact));
}
