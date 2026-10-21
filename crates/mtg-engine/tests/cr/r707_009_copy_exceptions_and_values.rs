//! CR 707.9, 707.5, 614.12: a copy effect's exceptions belong to that copy effect, and a
//! copy's copiable values are fixed when the copy is determined. Found by the copy property
//! (`mtg_engine::properties::copies`, docs/PROPERTIES.md).

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn triggered_abilities(t: &TestGame, id: ObjectId) -> usize {
    t.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Triggered(_)))
        .count()
}

/// The exceptions of a Clone-like card's "enter as a copy ..., except ..." apply only to
/// the copy effect that ability creates: a token copy of Callidus Assassin that copies
/// nothing as it enters, or a Clone that enters as a copy of it and declines the copied
/// choice, has none of them (no "When this creature enters, destroy ..." ability).
#[test]
fn a_copy_that_declines_its_own_copy_choice_gets_none_of_its_exceptions() {
    cr!("707.9a", "707.5", "614.12");
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Callidus Assassin");
    assert_eq!(triggered_abilities(&t, assassin), 0);
    t.lands(P0, "Island", 4);
    let cc = t.hand(P0, "Cackling Counterpart");
    // The token may enter as a copy of a creature (its copied ability): it doesn't.
    t.answer_choose(P0, &[]);
    t.cast(P0, cc).target(assassin).go();
    t.resolve_all();
    let all = t.named_on_battlefield("Callidus Assassin");
    let token = *all
        .iter()
        .find(|id| t.obj(**id).is_token())
        .expect("the token");
    assert_eq!(triggered_abilities(&t, token), 0);
    // "Enter tapped as a copy": only if it copies something.
    assert!(!t.obj(token).tapped);
    // A Clone entering as a copy of it, declining the copied "enter as a copy" choice.
    t.answer_choose(P1, &[Entity::Object(assassin)]);
    t.answer_choose(P1, &[]);
    let c = t.enter(P1, "Clone");
    let c = t.g.current(c);
    assert_eq!(t.obj(c).chars.name.as_str(), "Callidus Assassin");
    assert_eq!(triggered_abilities(&t, c), 0);
}

/// A token created as a copy has the copiable values of the object as the token is
/// created: an "as this enters" ability of the copy that changes the original (Ixidron
/// turning it face down) doesn't change what the token is.
#[test]
fn a_token_copy_copies_the_original_as_it_was_when_the_token_was_created() {
    cr!("707.2", "707.5", "614.12a");
    interp!("copy-values-fixed-when-determined");
    let mut t = TestGame::new(2);
    // A face-down creature, so Ixidron (power and toughness equal to the number of
    // face-down creatures) survives.
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, bears));
    let ixidron = t.battlefield(P0, "Ixidron");
    t.lands(P0, "Island", 4);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(ixidron).go();
    t.resolve_all();
    // The original was turned face down as the token entered. the Bears were turned face down as the token entered.
    assert!(t.obj(ixidron).face_down);
    let tokens: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| t.obj(*id).is_token())
            .collect();
    assert_eq!(tokens.len(), 1);
    let tok = t.obj(tokens[0]);
    assert!(!tok.face_down);
    assert_eq!(tok.chars.name.as_str(), "Ixidron");
    // Its power and toughness are the number of face-down creatures.
    assert_eq!(t.pt(tokens[0]), (2, 2));
    assert_eq!(t.pt(ixidron), (2, 2));
}

/// The same for a permanent entering as a copy of a chosen object: it copies the object
/// as it was chosen.
#[test]
fn a_clone_copies_the_chosen_object_as_it_was_chosen() {
    cr!("707.5", "614.12a");
    interp!("copy-values-fixed-when-determined");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, bears));
    let ixidron = t.battlefield(P0, "Ixidron");
    t.answer_choose(P1, &[Entity::Object(ixidron)]);
    let c = t.enter(P1, "Clone");
    let c = t.g.current(c);
    assert!(t.obj(ixidron).face_down);
    assert_eq!(t.obj(c).chars.name.as_str(), "Ixidron");
}
