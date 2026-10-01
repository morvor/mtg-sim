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
