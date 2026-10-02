//! Rulings batch P021 — a permanent that enters as a copy of a creature chooses as it
//! enters, so it can't copy a creature entering at the same time (CR 614.12, 707.2);
//! Renegade Doppelganger, which becomes a copy after entering, can (CR 603.6a), and its
//! copy effect ends in the cleanup step as damage is removed (CR 514.2).

use crate::r_p021_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::damage;
use crate::r_s24_common::enter_together;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn callidus_assassin_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Callidus Assassin",
        "If Callidus Assassin somehow enters the battlefield at the same time as another creature, Callidus Assassin can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Callidus Assassin", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn clone_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Clone",
        "If Clone somehow enters at the same time as another creature, Clone can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Clone", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn copycrook_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Copycrook",
        "If Copycrook somehow enters the battlefield at the same time as another creature, Copycrook can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Copycrook", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn dack_s_duplicate_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Dack's Duplicate",
        "If Dack's Duplicate somehow enters the battlefield at the same time as another creature, Dack's Duplicate can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Dack's Duplicate", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn evil_twin_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Evil Twin",
        "If Evil Twin somehow enters the battlefield at the same time as another creature, Evil Twin can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Evil Twin", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn gigantoplasm_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Gigantoplasm",
        "If Gigantoplasm somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Gigantoplasm", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn glasspool_mimic_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Glasspool Mimic // Glasspool Shore",
        "If Glasspool Mimic somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it(
        "Glasspool Mimic // Glasspool Shore",
        P0,
        "Grizzly Bears",
        P0,
        "Hill Giant",
    );
}

#[test]
fn machine_god_s_effigy_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Machine God's Effigy",
        "If Machine God’s Effigy somehow enters the battlefield at the same time as another creature, it can’t become a copy of that creature"
    );
    cant_copy_what_enters_with_it(
        "Machine God's Effigy",
        P0,
        "Grizzly Bears",
        P1,
        "Hill Giant",
    );
}

#[test]
fn malleable_impostor_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Malleable Impostor",
        "If Malleable Impostor somehow enters the battlefield at the same time as another creature, Malleable Impostor can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Malleable Impostor", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn mercurial_pretender_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Mercurial Pretender",
        "If Mercurial Pretender somehow enters the battlefield at the same time as another creature, Mercurial Pretender can’t become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Mercurial Pretender", P0, "Grizzly Bears", P0, "Hill Giant");
}

#[test]
fn mirror_image_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Mirror Image",
        "If Mirror Image somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Mirror Image", P0, "Grizzly Bears", P0, "Hill Giant");
}

#[test]
fn mirrorhall_mimic_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "If Mirrorhall Mimic somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it(
        "Mirrorhall Mimic // Ghastly Mimicry",
        P0,
        "Grizzly Bears",
        P1,
        "Hill Giant",
    );
}

#[test]
fn mocking_doppelganger_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Mocking Doppelganger",
        "If Mocking Doppelganger somehow enters the battlefield at the same time as another creature an opponent controls, it can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it(
        "Mocking Doppelganger",
        P0,
        "Grizzly Bears",
        P1,
        "Hill Giant",
    );
}

#[test]
fn phantasmal_image_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Phantasmal Image",
        "If Phantasmal Image somehow enters the battlefield at the same time as another creature, Phantasmal Image can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Phantasmal Image", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn pirated_copy_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Pirated Copy",
        "If Pirated Copy somehow enters the battlefield at the same time as another creature, Pirated Copy can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Pirated Copy", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn progenitor_mimic_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Progenitor Mimic",
        "If Progenitor Mimic somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it("Progenitor Mimic", P0, "Grizzly Bears", P1, "Hill Giant");
}

#[test]
fn quicksilver_gargantuan_cant_copy_a_creature_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Quicksilver Gargantuan",
        "If Quicksilver Gargantuan somehow enters at the same time as another creature (due to Mass Polymorph or Liliana Vess's third ability, for example), Quicksilver Gargantuan can't become a copy of that creature"
    );
    cant_copy_what_enters_with_it(
        "Quicksilver Gargantuan",
        P0,
        "Grizzly Bears",
        P1,
        "Hill Giant",
    );
}

#[test]
fn renegade_doppelganger_can_copy_a_creature_entering_at_the_same_time() {
    cr!("603.6a", "707.2");
    ruling!(
        "Renegade Doppelganger",
        "If Renegade Doppelganger and another creature enter under your control at the same time, Renegade Doppelganger's ability will trigger."
    );
    supported("Renegade Doppelganger");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let entered = enter_together(&mut t, &[(P0, "Renegade Doppelganger"), (P0, "Hill Giant")]);
    t.resolve_all();
    assert_eq!(t.obj_now(entered[0]).chars.name, "Hill Giant");
    assert_eq!(t.pt(entered[0]), (3, 3));
}

#[test]
fn renegade_doppelgangers_copy_effect_ends_as_damage_is_removed() {
    cr!("514.2", "707.2");
    ruling!(
        "Renegade Doppelganger",
        "During the cleanup step, Renegade Doppelganger's copy effect wears off."
    );
    supported("Renegade Doppelganger");
    let mut t = TestGame::new(2);
    let dopp = t.battlefield(P0, "Renegade Doppelganger");
    t.answer_yes(P0, true);
    let giant = t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.obj_now(dopp).chars.name, "Hill Giant");
    // 2 damage on a 3/3: it survives, and still does when it goes back to a 0/1.
    damage(&mut t, giant, 2, dopp);
    assert!(t.on_battlefield(dopp));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(dopp));
    let o = t.obj_now(dopp);
    assert_eq!(o.chars.name, "Renegade Doppelganger");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert_eq!(t.pt(dopp), (0, 1));
    assert_eq!(o.damage, 0);
    assert_eq!(
        abilities_with(&t, dopp, "Whenever another creature you control enters"),
        1
    );
}
