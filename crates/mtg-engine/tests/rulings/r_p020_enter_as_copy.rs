//! Rulings batch P020 — a creature that enters as a copy of a creature has the copied
//! creature's "enters" triggered abilities (they trigger as it enters) and its "as this
//! enters" / "enters with" replacement abilities (they apply as it enters) (CR 707.2,
//! 614.1c, 614.12, 603.6a).

use crate::r_p020_common::*;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `clone` enters for P0 as a copy of an Aven Riftwatcher controlled by `owner`: it has
/// three time counters (vanishing, an "enters with" ability) and its "enters" trigger
/// gains P0 2 life. Returns the clone.
fn clone_riftwatcher(clone: &str, owner: PlayerId) -> (TestGame, ObjectId) {
    supported(clone);
    supported(RIFTWATCHER);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(owner, RIFTWATCHER);
    let life = t.life(P0);
    t.answer_choose(P0, &[Entity::Object(rift)]);
    let c = t.enter(P0, clone);
    t.resolve_all();
    entered_as_riftwatcher(&t, c);
    assert_eq!(t.life(P0), life + 2, "{clone}: the copied ETB didn't trigger");
    // The original Riftwatcher is untouched.
    assert_eq!(t.counters(rift, counters::TIME), 0);
    (t, c)
}

#[test]
fn gigantoplasm_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Gigantoplasm",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Gigantoplasm enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    clone_riftwatcher("Gigantoplasm", P1);
}

#[test]
fn mirrorhall_mimic_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Mirrorhall Mimic enters the battlefield."
    );
    let (t, c) = clone_riftwatcher("Mirrorhall Mimic // Ghastly Mimicry", P1);
    assert!(t.obj_now(c).chars.has_subtype("Spirit"));
}

#[test]
fn phantasmal_image_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Phantasmal Image",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Phantasmal Image enters the battlefield."
    );
    let (t, c) = clone_riftwatcher("Phantasmal Image", P1);
    assert!(t.obj_now(c).chars.has_subtype("Illusion"));
}

#[test]
fn progenitor_mimic_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Progenitor Mimic",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Progenitor Mimic enters the battlefield."
    );
    clone_riftwatcher("Progenitor Mimic", P1);
}

#[test]
fn stunt_double_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Stunt Double",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Stunt Double enters the battlefield."
    );
    clone_riftwatcher("Stunt Double", P1);
}

#[test]
fn synth_infiltrator_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Synth Infiltrator",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Synth Infiltrator enters the battlefield."
    );
    let (t, c) = clone_riftwatcher("Synth Infiltrator", P1);
    assert!(t.obj_now(c).is(CardType::Artifact));
    assert!(t.obj_now(c).chars.has_subtype("Synth"));
}

#[test]
fn vizier_of_many_faces_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Vizier of Many Faces",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Vizier of Many Faces enters the battlefield."
    );
    clone_riftwatcher("Vizier of Many Faces", P1);
}

#[test]
fn copycrook_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Copycrook",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Copycrook enters the battlefield."
    );
    clone_riftwatcher("Copycrook", P1);
}

#[test]
fn mercurial_pretender_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Mercurial Pretender",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Mercurial Pretender enters the battlefield."
    );
    clone_riftwatcher("Mercurial Pretender", P0);
}

#[test]
fn glasspool_mimic_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Glasspool Mimic // Glasspool Shore",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Glasspool Mimic enters the battlefield."
    );
    let (t, c) = clone_riftwatcher("Glasspool Mimic // Glasspool Shore", P0);
    assert!(t.obj_now(c).chars.has_subtype("Rogue"));
}

#[test]
fn mirror_image_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Mirror Image",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Mirror Image enters the battlefield."
    );
    clone_riftwatcher("Mirror Image", P0);
}

#[test]
fn visage_bandit_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Visage Bandit",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Visage Bandit enters the battlefield."
    );
    let (t, c) = clone_riftwatcher("Visage Bandit", P0);
    assert!(t.obj_now(c).chars.has_subtype("Shapeshifter"));
}

#[test]
fn jwari_shapeshifter_has_the_copied_allys_enters_abilities() {
    cr!("707.2", "603.6a");
    ruling!(
        "Jwari Shapeshifter",
        "Any \"enters\" abilities of the copied creature will trigger when Jwari Shapeshifter enters."
    );
    supported("Jwari Shapeshifter");
    supported("Kyoshi Warriors");
    // Kyoshi Warriors: "When this creature enters, create a 1/1 white Ally creature token."
    let mut t = TestGame::new(2);
    let warriors = t.battlefield(P1, "Kyoshi Warriors");
    t.answer_choose(P0, &[Entity::Object(warriors)]);
    let c = t.enter(P0, "Jwari Shapeshifter");
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name, "Kyoshi Warriors");
    let toks = new_tokens_of(&t, P0, &[]);
    assert_eq!(toks.len(), 1);
    assert!(t.obj_now(toks[0]).chars.has_subtype("Ally"));
}

#[test]
fn quicksilver_gargantuan_has_the_copied_enters_abilities_and_is_7_7() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Quicksilver Gargantuan",
        "Any \"enters\" abilities of the copied creature will trigger when Quicksilver Gargantuan enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature will also work."
    );
    // "..., except it's 7/7."
    let (t, c) = clone_riftwatcher("Quicksilver Gargantuan", P1);
    assert_eq!(t.pt(c), (7, 7));
}

#[test]
fn sakashimas_student_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Sakashima's Student",
        "Any \"enters\" abilities of the copied creature will trigger when Sakashima's Student enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature (like devour) will also work."
    );
    // "..., except it's a Ninja in addition to its other creature types."
    let (t, c) = clone_riftwatcher("Sakashima's Student", P1);
    assert!(t.obj_now(c).chars.has_subtype("Ninja"));
    assert!(t.obj_now(c).chars.has_subtype("Bird"));
}

#[test]
fn sakashimas_student_copying_a_devour_creature_devours() {
    cr!("707.2", "702.82a", "614.1c");
    ruling!(
        "Sakashima's Student",
        "Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature (like devour) will also work."
    );
    // Gorger Wurm: 5/5, devour 1 ("As this creature enters, you may sacrifice any number
    // of creatures. It enters with that many +1/+1 counters on it.").
    supported("Gorger Wurm");
    supported("Sakashima's Student");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Gorger Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(elves)]);
    let c = t.enter(P0, "Sakashima's Student");
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name, "Gorger Wurm");
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(elves));
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert_eq!(t.pt(c), (7, 7));
}

#[test]
fn malleable_impostor_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Malleable Impostor",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when Malleable Impostor enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    // "... a copy of a creature an opponent controls, except it's a Faerie Shapeshifter in
    // addition to its other types and it has flying."
    let (t, c) = clone_riftwatcher("Malleable Impostor", P1);
    let o = t.obj_now(c);
    assert!(o.chars.has_subtype("Faerie") && o.chars.has_subtype("Shapeshifter"));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn deceptive_frostkite_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Deceptive Frostkite",
        "Any enters abilities of the copied creature will trigger when Deceptive Frostkite enters. Any “as [this creature] enters” or “[this creature] enters with” abilities of the copied permanent will also work."
    );
    // "... a copy of a creature you control with power 4 or greater, except it's a Dragon
    // in addition to its other types and it has flying." Giant Growth makes P0's Aven
    // Riftwatcher 5/6, so it can be chosen; the copy is a 2/3 (copiable values only).
    supported("Deceptive Frostkite");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    crate::r_s25_common::cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(rift)]);
    t.resolve_all();
    let life = t.life(P0);
    t.answer_choose(P0, &[Entity::Object(rift)]);
    let c = t.enter(P0, "Deceptive Frostkite");
    t.resolve_all();
    entered_as_riftwatcher(&t, c);
    assert_eq!(t.life(P0), life + 2);
    assert_eq!(t.pt(c), (2, 3));
    assert!(t.obj_now(c).chars.has_subtype("Dragon"));
}

#[test]
fn lazotep_convert_has_the_copied_creature_cards_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a", "712.14a");
    ruling!(
        "Invasion of Amonkhet // Lazotep Convert",
        "Any enters-the-battlefield abilities of the copied creature card will trigger when Lazotep Convert enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the chosen creature card will also work."
    );
    // "You may have this creature enter as a copy of any creature card in a graveyard,
    // except it's a 4/4 black Zombie in addition to its other colors and types."
    supported("Invasion of Amonkhet // Lazotep Convert");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, RIFTWATCHER);
    let life = t.life(P0);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let c = crate::r_s17_common::enter_transformed(&mut t, P0, "Invasion of Amonkhet // Lazotep Convert");
    t.resolve_all();
    entered_as_riftwatcher(&t, c);
    assert_eq!(t.life(P0), life + 2);
    let o = t.obj_now(c);
    assert_eq!(t.pt(c), (4, 4));
    assert!(o.chars.has_subtype("Zombie") && o.chars.has_subtype("Bird"));
    assert!(o.chars.colors.contains(Color::Black) && o.chars.colors.contains(Color::White));
    // The card stays in the graveyard.
    assert!(t.in_graveyard(P1, RIFTWATCHER));
}

#[test]
fn dacks_duplicate_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Dack's Duplicate",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Dack's Duplicate enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    clone_riftwatcher("Dack's Duplicate", P1);
}

#[test]
fn dacks_duplicate_copies_printed_values_plus_haste_and_dethrone() {
    cr!("707.2", "707.9a", "702.105a");
    ruling!(
        "Dack's Duplicate",
        "Dack's Duplicate copies exactly what was printed on the original creature and nothing more (unless that creature is copying something else or is a token; see below), except it will have haste and dethrone. It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    use mtg_engine::keywords::KeywordKind;
    supported("Dack's Duplicate");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P1, "Holy Strength", giant);
    crate::r_s26_common::dress_up(&mut t, giant);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let c = t.enter(P0, "Dack's Duplicate");
    t.settle();
    let o = t.obj_now(c);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.contains(Color::Red) && !o.chars.colors.contains(Color::Green));
    assert!(o.has_keyword(KeywordKind::Haste) && o.has_keyword(KeywordKind::Dethrone));
    assert_eq!(t.pt(c), (3, 3));
    assert!(crate::r_s26_common::fresh(&t, c));
}
