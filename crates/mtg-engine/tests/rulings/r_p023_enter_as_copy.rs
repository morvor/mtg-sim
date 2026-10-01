//! Rulings batch P023 — permanents that enter as a copy of a creature ("You may have this
//! creature enter as a copy of ..."): choosing a token copies the original
//! characteristics the effect that created the token gave it, and the permanent doesn't
//! become a token (CR 707.2, 111.4); choosing something that's copying something else
//! copies what it copies, as modified by that copy effect's exceptions (CR 707.3,
//! 707.9b); a copied {X} is 0 (CR 107.3i, 202.3).

use crate::r_p023_common::*;
use crate::r_s01_common::supported;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s `copier` enters copying a dressed-up Wolf token controlled by `wolf_owner`; the
/// copier is a nontoken copy of the Wolf's original characteristics.
fn copies_wolf(copier: &str, p: PlayerId, wolf_owner: PlayerId, pt: bool) -> (TestGame, ObjectId) {
    supported(copier);
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, wolf_owner);
    let c = enter_copying(&mut t, p, copier, wolf);
    assert_wolf(&t, c, false, pt);
    assert!(!t.obj_now(c).tapped);
    (t, c)
}

/// `p`'s `copier` enters copying a Clone that's copying Serra Angel (both controlled by
/// `clone_owner`); the copier is a Serra Angel.
fn copies_cloned_angel(
    copier: &str,
    p: PlayerId,
    clone_owner: PlayerId,
    pt: bool,
) -> (TestGame, ObjectId) {
    supported(copier);
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, clone_owner);
    let c = enter_copying(&mut t, p, copier, clone);
    assert_angel(&t, c, pt);
    assert!(!t.obj_now(c).is_token());
    (t, c)
}

fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// The number of abilities the object has whose text contains `s`.
fn abilities_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(s))
        .count()
}

// --- Gigantoplasm ---------------------------------------------------------------------

#[test]
fn gigantoplasm_copying_a_token() {
    cr!("707.2", "111.4", "707.9a");
    ruling!(
        "Gigantoplasm",
        "If the chosen creature is a token, Gigantoplasm copies the original characteristics of that token as stated by the effect that created the token. Gigantoplasm isn't a token, even if it's copying one."
    );
    let (t, c) = copies_wolf("Gigantoplasm", P0, P1, true);
    assert_eq!(abilities_with(&t, c, "base power and toughness X/X"), 1);
}

#[test]
fn gigantoplasm_copying_another_gigantoplasm() {
    cr!("707.3", "707.9b");
    ruling!(
        "Gigantoplasm",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Gigantoplasm), then Gigantoplasm enters the battlefield as whatever the chosen creature was copying (plus any abilities it gained as part of the copy process, if applicable)."
    );
    supported("Gigantoplasm");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let first = enter_copying(&mut t, P1, "Gigantoplasm", angel);
    assert_angel(&t, first, true);
    let second = enter_copying(&mut t, P0, "Gigantoplasm", first);
    assert_angel(&t, second, true);
    // The ability the first one gained is part of its copiable values; the second gets
    // it again from its own exception.
    assert_eq!(abilities_with(&t, second, "base power and toughness X/X"), 2);
}

// --- Glasspool Mimic, Visage Bandit ------------------------------------------------------

#[test]
fn glasspool_mimic_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Glasspool Mimic // Glasspool Shore",
        "If the chosen creature is a token, Glasspool Mimic copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Glasspool Mimic doesn't become a token in this case."
    );
    let (t, c) = copies_wolf("Glasspool Mimic // Glasspool Shore", P0, P0, true);
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

#[test]
fn glasspool_mimic_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Glasspool Mimic // Glasspool Shore",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Glasspool Mimic), then Glasspool Mimic enters the battlefield as whatever the chosen creature copied."
    );
    let (t, c) = copies_cloned_angel("Glasspool Mimic // Glasspool Shore", P0, P0, true);
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

#[test]
fn visage_bandit_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Visage Bandit",
        "If the copied creature is a token, Visage Bandit copies the original characteristics of that token as stated by the effect that created that token, with the stated exceptions."
    );
    let (t, c) = copies_wolf("Visage Bandit", P0, P0, true);
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

#[test]
fn visage_bandit_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Visage Bandit",
        "If the copied creature is copying something else, then Visage Bandit enters the battlefield as whatever that creature copied, with the stated exceptions."
    );
    let (t, c) = copies_cloned_angel("Visage Bandit", P0, P0, true);
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

// --- Mercurial Pretender -----------------------------------------------------------------

#[test]
fn mercurial_pretender_copying_a_token() {
    cr!("707.2", "111.4", "707.9a");
    ruling!(
        "Mercurial Pretender",
        "If the chosen creature is a token, Mercurial Pretender copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Mercurial Pretender is not a token, even when copying one."
    );
    let (t, c) = copies_wolf("Mercurial Pretender", P0, P0, true);
    assert_eq!(abilities_with(&t, c, "Return"), 1);
}

#[test]
fn mercurial_pretender_copying_another_mercurial_pretender() {
    cr!("707.3", "707.9b");
    ruling!(
        "Mercurial Pretender",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Mercurial Pretender), then Mercurial Pretender enters the battlefield as whatever the chosen creature is copying (plus any abilities it gained as part of the copying process, if applicable)."
    );
    supported("Mercurial Pretender");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let first = enter_copying(&mut t, P0, "Mercurial Pretender", angel);
    let second = enter_copying(&mut t, P0, "Mercurial Pretender", first);
    assert_angel(&t, second, true);
    assert_eq!(abilities_with(&t, second, "Return"), 2);
}

// --- Mirror Image, Stunt Double, Clone, Vizier of Many Faces, Progenitor Mimic ---------

#[test]
fn mirror_image_copying_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Mirror Image",
        "If the chosen creature is a token, Mirror Image copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Mirror Image doesn't become a token in this case."
    );
    copies_wolf("Mirror Image", P0, P0, true);
}

#[test]
fn mirror_image_copying_a_clone() {
    cr!("707.3");
    ruling!(
        "Mirror Image",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Mirror Image), then Mirror Image enters the battlefield as whatever the chosen creature copied."
    );
    copies_cloned_angel("Mirror Image", P0, P0, true);
}

#[test]
fn stunt_double_copying_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Stunt Double",
        "If the chosen creature is a token, Stunt Double copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Stunt Double isn’t a token."
    );
    copies_wolf("Stunt Double", P0, P1, true);
}

#[test]
fn stunt_double_copying_a_clone() {
    cr!("707.3");
    ruling!(
        "Stunt Double",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Stunt Double), then Stunt Double enters the battlefield as whatever the chosen creature copied."
    );
    copies_cloned_angel("Stunt Double", P0, P1, true);
}

#[test]
fn clone_copying_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Clone",
        "If the copied creature is a token, Clone copies the original characteristics of that token as stated by the effect that created the token."
    );
    copies_wolf("Clone", P0, P1, true);
}

#[test]
fn clone_copying_a_clone() {
    cr!("707.3");
    ruling!(
        "Clone",
        "If the copied creature is copying something else (for example, if the copied creature is an Evil Twin), then Clone enters as whatever that creature copied."
    );
    copies_cloned_angel("Clone", P0, P1, true);
}

#[test]
fn vizier_of_many_faces_copying_a_clone() {
    cr!("707.3");
    ruling!(
        "Vizier of Many Faces",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Vizier of Many Faces), then Vizier of Many Faces enters the battlefield as whatever the chosen creature copied."
    );
    copies_cloned_angel("Vizier of Many Faces", P0, P1, true);
}

#[test]
fn progenitor_mimic_copying_a_token() {
    cr!("707.2", "111.4", "707.9a");
    ruling!(
        "Progenitor Mimic",
        "If the chosen creature is a token, Progenitor Mimic copies the original characteristics of that token as stated by the effect that put it onto the battlefield. Copying a token doesn't make Progenitor Mimic become a token."
    );
    let (t, c) = copies_wolf("Progenitor Mimic", P0, P1, true);
    assert_eq!(abilities_with(&t, c, "isn't a token"), 1);
}

// --- Mirrorhall Mimic, Phantasmal Image, Sakashima's Student, Synth Infiltrator -------

#[test]
fn mirrorhall_mimic_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "If the chosen creature is a token, Mirrorhall Mimic copies the original characteristics of that token as stated by the effect that put the token onto the battlefield, except it's a Spirit. Mirrorhall Mimic doesn't become a token in this case."
    );
    let (t, c) = copies_wolf("Mirrorhall Mimic // Ghastly Mimicry", P0, P1, true);
    assert!(has_subtype(&t, c, "Spirit"));
}

#[test]
fn mirrorhall_mimic_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Mirrorhall Mimic), then Mirrorhall Mimic enters the battlefield as whatever the chosen creature copied."
    );
    let (t, c) = copies_cloned_angel("Mirrorhall Mimic // Ghastly Mimicry", P0, P1, true);
    assert!(has_subtype(&t, c, "Spirit"));
}

#[test]
fn phantasmal_image_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Phantasmal Image",
        "If the chosen creature is a token, Phantasmal Image copies the original characteristics of that token as stated by the effect that created the token. Phantasmal Image is not a token in this case."
    );
    let (t, c) = copies_wolf("Phantasmal Image", P0, P1, true);
    assert!(has_subtype(&t, c, "Illusion"));
}

#[test]
fn phantasmal_image_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Phantasmal Image",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Phantasmal Image), then Phantasmal Image enters the battlefield as whatever the chosen creature copied."
    );
    let (t, c) = copies_cloned_angel("Phantasmal Image", P0, P1, true);
    assert!(has_subtype(&t, c, "Illusion"));
}

#[test]
fn sakashimas_student_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Sakashima's Student",
        "If the chosen creature is a token, Sakashima's Student copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Sakashima's Student is not a token."
    );
    let (t, c) = copies_wolf("Sakashima's Student", P0, P1, true);
    assert!(has_subtype(&t, c, "Ninja"));
}

#[test]
fn sakashimas_student_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Sakashima's Student",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Sakashima's Student), then your Sakashima's Student enters as whatever the chosen creature copied."
    );
    let (t, c) = copies_cloned_angel("Sakashima's Student", P0, P1, true);
    assert!(has_subtype(&t, c, "Ninja") && has_subtype(&t, c, "Angel"));
}

#[test]
fn synth_infiltrator_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Synth Infiltrator",
        "If the copied creature is a token, Synth Infiltrator copies the original characteristics of that token as stated by the effect that created that token, with the noted exceptions."
    );
    let (t, c) = copies_wolf("Synth Infiltrator", P0, P1, true);
    assert!(has_subtype(&t, c, "Synth") && t.obj_now(c).is(CardType::Artifact));
}

#[test]
fn synth_infiltrator_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Synth Infiltrator",
        "If the copied creature is copying something else, then Synth Infiltrator enters the battlefield as whatever that creature copied, with the noted exceptions."
    );
    let (t, c) = copies_cloned_angel("Synth Infiltrator", P0, P1, true);
    assert!(has_subtype(&t, c, "Synth") && t.obj_now(c).is(CardType::Artifact));
}

// --- Copycrook, Dack's Duplicate ----------------------------------------------------------

#[test]
fn copycrook_copying_a_token() {
    cr!("707.2", "111.4", "707.9a");
    ruling!(
        "Copycrook",
        "If the chosen permanent is a token, Copycrook copies the original characteristics of that token as stated by the effect that put the token onto the battlefield, with the listed exception. Copycrook does not become a token."
    );
    let (t, c) = copies_wolf("Copycrook", P0, P1, true);
    assert_eq!(abilities_with(&t, c, "connives"), 1);
}

#[test]
fn copycrook_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Copycrook",
        "If the chosen creature is copying something else (for example, if the chosen creature is a Clone), then your Copycrook enters the battlefield as whatever the chosen creature copied, with the listed exception."
    );
    let (t, c) = copies_cloned_angel("Copycrook", P0, P1, true);
    assert_eq!(abilities_with(&t, c, "connives"), 1);
}

#[test]
fn dacks_duplicate_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Dack's Duplicate",
        "If the chosen creature is a token, Dack's Duplicate copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Dack's Duplicate is not a token, even when copying one."
    );
    let (t, c) = copies_wolf("Dack's Duplicate", P0, P1, true);
    assert!(has_kw(&t, c, KeywordKind::Haste) && has_kw(&t, c, KeywordKind::Dethrone));
}

#[test]
fn dacks_duplicate_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Dack's Duplicate",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Dack's Duplicate), then Dack's Duplicate enters the battlefield as whatever the chosen creature is copying. It will have haste and dethrone."
    );
    let (t, c) = copies_cloned_angel("Dack's Duplicate", P0, P1, true);
    assert!(has_kw(&t, c, KeywordKind::Haste) && has_kw(&t, c, KeywordKind::Dethrone));
}

// --- Quicksilver Gargantuan, Malleable Impostor, Deceptive Frostkite ---------------------

#[test]
fn quicksilver_gargantuan_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Quicksilver Gargantuan",
        "If the chosen creature is a token, your Quicksilver Gargantuan copies the original characteristics of that token as stated by the effect that put the token onto the battlefield, except for its power and toughness. Your Quicksilver Gargantuan is not a token."
    );
    let (t, c) = copies_wolf("Quicksilver Gargantuan", P0, P1, false);
    assert_eq!(t.pt(c), (7, 7));
}

#[test]
fn quicksilver_gargantuan_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Quicksilver Gargantuan",
        "If the chosen creature is copying something else (for example, if the chosen creature is a Clone), then your Quicksilver Gargantuan enters as whatever the chosen creature copied, except for its power and toughness."
    );
    let (t, c) = copies_cloned_angel("Quicksilver Gargantuan", P0, P1, false);
    assert_eq!(t.pt(c), (7, 7));
}

#[test]
fn malleable_impostor_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Malleable Impostor",
        "If the chosen permanent is a token, Malleable Impostor copies the original characteristics of that token as stated by the effect that put the token onto the battlefield, with the listed exceptions."
    );
    let (t, c) = copies_wolf("Malleable Impostor", P0, P1, true);
    assert!(has_subtype(&t, c, "Faerie") && has_subtype(&t, c, "Shapeshifter"));
    assert!(has_kw(&t, c, KeywordKind::Flying));
}

#[test]
fn malleable_impostor_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Malleable Impostor",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is a Clone), then your Malleable Impostor enters the battlefield as whatever the chosen permanent copied, with the stated exceptions."
    );
    let (t, c) = copies_cloned_angel("Malleable Impostor", P0, P1, true);
    assert!(has_subtype(&t, c, "Faerie") && has_subtype(&t, c, "Angel"));
}

#[test]
fn deceptive_frostkite_copying_a_token() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Deceptive Frostkite",
        "If the copied creature is a token, Deceptive Frostkite copies the original characteristics of that token as stated by the effect that created the token, with the listed exceptions."
    );
    // The Wolf has power 4 or greater (7) while it's chosen; the copy is a 2/2.
    let (t, c) = copies_wolf("Deceptive Frostkite", P0, P0, true);
    assert!(has_subtype(&t, c, "Dragon") && has_kw(&t, c, KeywordKind::Flying));
}

#[test]
fn deceptive_frostkite_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Deceptive Frostkite",
        "If the copied creature is copying something else, then Deceptive Frostkite enters as whatever that creature copied, with the listed exceptions."
    );
    let (t, c) = copies_cloned_angel("Deceptive Frostkite", P0, P0, true);
    assert!(has_subtype(&t, c, "Dragon") && has_subtype(&t, c, "Angel"));
}

// --- Jwari Shapeshifter (an Ally) ---------------------------------------------------------

#[test]
fn jwari_shapeshifter_copying_an_ally_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Jwari Shapeshifter",
        "If the chosen creature is a token, your Jwari Shapeshifter copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Your Jwari Shapeshifter is not a token."
    );
    supported("Jwari Shapeshifter");
    let mut t = TestGame::new(2);
    let ally = token_of(
        &mut t,
        P1,
        TokenSpec {
            name: "Kor Ally".into(),
            colors: ColorSet::single(Color::White),
            supertypes: vec![],
            card_types: vec![CardType::Creature],
            subtypes: vec!["Kor".into(), "Ally".into()],
            power: Some(1),
            toughness: Some(1),
            abilities: vec![],
            scryfall_name: None,
        },
    );
    crate::r_s26_common::dress_up(&mut t, ally);
    let c = enter_copying(&mut t, P0, "Jwari Shapeshifter", ally);
    let o = t.obj_now(c);
    assert_eq!(o.chars.name, "Kor Ally");
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert!(!o.is_token());
    assert_eq!(t.pt(c), (1, 1));
}

#[test]
fn jwari_shapeshifter_copying_a_clone_of_an_ally() {
    cr!("707.3");
    ruling!(
        "Jwari Shapeshifter",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Jwari Shapeshifter), then your Jwari Shapeshifter enters as whatever the chosen creature copied."
    );
    supported("Jwari Shapeshifter");
    supported("Affa Protector");
    let mut t = TestGame::new(2);
    let affa = t.battlefield(P1, "Affa Protector");
    let clone = clone_of(&mut t, P1, affa);
    let c = enter_copying(&mut t, P0, "Jwari Shapeshifter", clone);
    assert_eq!(t.obj_now(c).chars.name, "Affa Protector");
    assert_eq!(t.pt(c), (1, 4));
    assert!(has_kw(&t, c, KeywordKind::Vigilance));
}

// --- {X} in the copied mana cost -------------------------------------------------------------

fn copy_of_hydra_has_x_zero(copier: &str) {
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let c = enter_copying(&mut t, P0, copier, hydra);
    assert_eq!(t.obj_now(c).chars.name, "Benevolent Hydra");
    assert_eq!(mv_now(&mut t, c), 2);
    assert_eq!(t.counters(c, counters::PLUS1), 0);
    assert_eq!(t.pt(c), (1, 1));
}

#[test]
fn stunt_double_copying_a_creature_with_x_in_its_cost() {
    cr!("107.3g", "107.3m", "202.3e", "707.2");
    ruling!(
        "Stunt Double",
        "If the copied creature has {X} in its mana cost, that X is considered to be 0."
    );
    supported("Stunt Double");
    copy_of_hydra_has_x_zero("Stunt Double");
}

#[test]
fn dacks_duplicate_copying_a_creature_with_x_in_its_cost() {
    cr!("107.3g", "107.3m", "202.3e", "707.2");
    ruling!(
        "Dack's Duplicate",
        "If the copied creature has {X} in its mana cost (such as Grenzo, Dungeon Warden), X is considered to be 0."
    );
    supported("Dack's Duplicate");
    copy_of_hydra_has_x_zero("Dack's Duplicate");
}

// --- Machine God's Effigy -----------------------------------------------------------------

/// Machine God's Effigy is a noncreature artifact (its only card type) with the copied
/// name, colors and abilities, plus "{T}: Add {U}" — which works the turn it enters.
fn assert_effigy(t: &mut TestGame, effigy: ObjectId, name: &str, color: Color) {
    let o = t.obj_now(effigy);
    assert_eq!(o.chars.name, name);
    assert!(o.is(CardType::Artifact) && !o.is(CardType::Creature));
    assert!(o.chars.subtypes.is_empty(), "{:?}", o.chars.subtypes);
    assert!(o.chars.colors.contains(color));
    assert!(!o.is_token() && !o.tapped);
    assert!(o.counters.values().all(|n| *n == 0));
    crate::r_s06_common::activate_containing(t, P0, effigy, "Add {U}").expect("mana ability");
    assert_eq!(t.g.player(P0).mana_pool.count(mtg_engine::mana::ManaType::U), 1);
}

#[test]
fn machine_gods_effigy_copying_a_token() {
    cr!("707.2", "111.4", "707.9b", "205.1a");
    ruling!(
        "Machine God's Effigy",
        "If the chosen creature is a token, Machine God’s Effigy copies the original characteristics of that token as stated by the effect that created the token, plus the listed exceptions. Machine God’s Effigy is not a token, even when copying one."
    );
    supported("Machine God's Effigy");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let effigy = enter_copying(&mut t, P0, "Machine God's Effigy", wolf);
    assert_effigy(&mut t, effigy, "Wolf", Color::Green);
    assert!(!t.obj_now(effigy).chars.colors.contains(Color::Blue));
}

#[test]
fn machine_gods_effigy_copying_a_clone() {
    cr!("707.3", "707.9b", "205.1a");
    ruling!(
        "Machine God's Effigy",
        "If the chosen creature is copying something else, then Machine God’s Effigy enters the battlefield as whatever the chosen creature is copying (with the listed exceptions)."
    );
    supported("Machine God's Effigy");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let effigy = enter_copying(&mut t, P0, "Machine God's Effigy", clone);
    assert_effigy(&mut t, effigy, "Serra Angel", Color::White);
    assert!(t.obj_now(effigy).has_keyword(KeywordKind::Flying));
}
