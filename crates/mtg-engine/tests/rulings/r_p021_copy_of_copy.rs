//! Rulings batch P021 — becoming a copy of something that's copying something else copies
//! what it copies, as modified by that copy effect's exceptions (CR 707.3, 707.9b);
//! becoming a copy of a token copies the token's original characteristics and doesn't
//! make the permanent a token (CR 707.2, 111.3).

use crate::r_p021_common::*;
use crate::r_p023_common::{assert_angel, assert_wolf, cloned_angel, dressed_wolf, enter_copying};
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s18_common::lands_for;
use crate::r_s24_common::choose_creature_type;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_copy_of_dacks_duplicate_has_haste_and_dethrone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Dack's Duplicate",
        "Haste and dethrone are part of the copiable values of Dack's Duplicate."
    );
    supported("Dack's Duplicate");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let dack = enter_copying(&mut t, P0, "Dack's Duplicate", angel);
    let clone = crate::r_p023_common::clone_of(&mut t, P0, dack);
    assert_angel(&t, clone, true);
    assert!(has_kw(&t, clone, KeywordKind::Haste));
    assert!(has_kw(&t, clone, KeywordKind::Dethrone));
}

#[test]
fn artisan_of_forms_copying_a_token_copies_its_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Artisan of Forms",
        "If Artisan of Forms becomes a copy of a token creature, it copies the original characteristics of that token"
    );
    supported("Artisan of Forms");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Artisan of Forms");
    let wolf = dressed_wolf(&mut t, P1);
    // Giant Growth targeting Artisan: heroic triggers and Artisan copies the Wolf.
    lands_for(&mut t, P0, "{G}");
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(artisan)).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wolf)]);
    t.resolve_all();
    assert_wolf(&t, artisan, false, false);
    // 2/2 plus Giant Growth.
    assert_eq!(t.pt(artisan), (5, 5));
    assert_eq!(abilities_with(&t, artisan, "spell that targets"), 1);
}

#[test]
fn cemetery_puca_copying_a_token_copies_its_original_characteristics() {
    cr!("707.2", "111.3", "603.10a");
    ruling!(
        "Cemetery Puca",
        "If Cemetery Puca becomes a copy of a token, it copies the original characteristics of that token"
    );
    supported("Cemetery Puca");
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    t.lands(P0, "Wastes", 1);
    let wolf = dressed_wolf(&mut t, P1);
    t.answer_yes(P0, true);
    destroy(&mut t, wolf);
    t.resolve_all();
    assert_wolf(&t, puca, false, true);
    assert_eq!(abilities_with(&t, puca, "Whenever a creature dies"), 1);
}

#[test]
fn mindlink_mech_copying_a_clone_copies_what_it_copies() {
    cr!("707.3", "707.9b", "702.122a");
    ruling!(
        "Mindlink Mech",
        "If Mindlink Mech becomes a copy of a creature that is copying something else, it becomes a copy of whatever that creature is a copy of."
    );
    supported("Mindlink Mech");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mindlink Mech");
    let clone = cloned_angel(&mut t, P0);
    t.answer_choose(P0, &[obj(clone)]);
    activate_containing(&mut t, P0, mech, "Crew").expect("crew");
    t.answer_targets(P0, &[obj(clone)]);
    t.resolve_all();
    let o = t.obj_now(mech);
    assert_eq!(o.chars.name, "Serra Angel");
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(has_subtype(&t, mech, "Vehicle") && has_subtype(&t, mech, "Angel"));
    assert!(has_kw(&t, mech, KeywordKind::Vigilance) && has_kw(&t, mech, KeywordKind::Flying));
    assert_eq!(t.pt(mech), (4, 3));
}

#[test]
fn mirage_mirror_copying_a_clone_copies_what_it_copies() {
    cr!("707.3");
    ruling!(
        "Mirage Mirror",
        "If Mirage Mirror copies a permanent that's copying something else, it will become whatever the target is copying."
    );
    supported("Mirage Mirror");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Mirage Mirror");
    let clone = cloned_angel(&mut t, P1);
    lands_for(&mut t, P0, "{2}");
    t.activate(P0, mirror, 0, &[obj(clone)]).expect("activate");
    t.resolve_all();
    assert_angel(&t, mirror, true);
}

#[test]
fn mirror_of_the_forebears_copying_a_clone_copies_what_it_copies() {
    cr!("707.3", "707.9b");
    ruling!(
        "Mirror of the Forebears",
        "If Mirror of the Forebears copies a creature that’s copying something else, it will become whatever the target is copying."
    );
    supported("Mirror of the Forebears");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Angel");
    let mirror = t.enter(P0, "Mirror of the Forebears");
    t.settle();
    let clone = cloned_angel(&mut t, P0);
    lands_for(&mut t, P0, "{1}");
    t.activate(P0, mirror, 0, &[obj(clone)]).expect("activate");
    t.resolve_all();
    assert_angel(&t, mirror, true);
    assert!(t.obj_now(mirror).is(CardType::Artifact));
}

#[test]
fn protean_thaumaturge_copying_a_clone_copies_what_it_copies() {
    cr!("707.3", "707.9b");
    ruling!(
        "Protean Thaumaturge",
        "If Protean Thaumaturge copies a creature that’s copying something else, it will become whatever the target is copying."
    );
    supported("Protean Thaumaturge");
    let mut t = TestGame::new(2);
    let thaum = t.battlefield(P0, "Protean Thaumaturge");
    let clone = cloned_angel(&mut t, P1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(clone)]);
    t.enter(P0, "Pacifism");
    t.resolve_all();
    assert_angel(&t, thaum, true);
    assert_eq!(
        abilities_with(&t, thaum, "enchantment you control enters"),
        1
    );
}
