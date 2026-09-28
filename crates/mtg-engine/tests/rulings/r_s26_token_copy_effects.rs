//! Rulings batch S26 — effects applied to token copies after they're created aren't part
//! of what later copies copy (CR 707.2, 611.2c), and token copies of face-down creatures
//! copy only the face-down characteristics (CR 708.2, 707.2).

use crate::r_s01_common::supported;
use crate::r_s06_common::attach_new;
use crate::r_s11_common::manifest_card;
use crate::r_s26_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_copy_of_a_hate_mirage_token_has_no_haste_and_isnt_exiled() {
    cr!("707.2", "611.2c", "603.7c");
    ruling!(
        "Hate Mirage",
        "Each token gains haste after it has been created. If something copies one of these tokens, the copy won't have haste, and you won't exile it at the beginning of the next end step."
    );
    supported("Hate Mirage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let mirage = t.hand(P0, "Hate Mirage");
    let before = t.g.battlefield.clone();
    t.cast(P0, mirage)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    assert!(toks
        .iter()
        .all(|x| t.obj_now(*x).has_keyword(KeywordKind::Haste)));
    let bears_token = *toks
        .iter()
        .find(|x| t.obj_now(**x).chars.name == "Grizzly Bears")
        .unwrap();
    // A Clone enters as a copy of the Grizzly Bears token: no haste.
    t.answer_choose(P0, &[Entity::Object(bears_token)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name, "Grizzly Bears");
    assert!(!t.obj(clone).has_keyword(KeywordKind::Haste));
    // At the beginning of the next end step, only Hate Mirage's tokens are exiled.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(toks.iter().all(|x| !t.g.is_live(*x)));
    assert!(t.on_battlefield(clone));
}

/// Whether the object is a face-up colorless 2/2 creature with no name, abilities or
/// creature types.
fn blank_2_2(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    !o.face_down
        && o.is(CardType::Creature)
        && t.pt(id) == (2, 2)
        && o.chars.colors.is_colorless()
        && o.chars.name.is_empty()
        && o.chars.abilities.is_empty()
        && o.chars.subtypes.is_empty()
}

#[test]
fn clone_legion_copies_a_face_down_creature_as_a_blank_2_2() {
    cr!("708.2", "707.2", "708.10");
    ruling!(
        "Clone Legion",
        "A token that enters the battlefield as a copy of a face-down creature is a face-up colorless 2/2 creature with no name, abilities, or creature types."
    );
    supported("Clone Legion");
    let mut t = TestGame::new(2);
    let fd = manifest_card(&mut t, P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 9);
    let legion = t.hand(P0, "Clone Legion");
    let before = t.g.battlefield.clone();
    t.cast(P0, legion).target(P1).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().any(|x| blank_2_2(&t, *x)));
    assert!(toks
        .iter()
        .any(|x| t.obj_now(*x).chars.name == "Grizzly Bears"));
    assert!(t.obj_now(fd).face_down && t.on_battlefield(bears));
}

#[test]
fn mirror_mockery_copies_a_face_down_attacker_as_a_blank_2_2() {
    cr!("708.2", "707.2", "603.7c");
    ruling!(
        "Mirror Mockery",
        "A token that enters the battlefield as a copy of a face-down creature is a face-up colorless 2/2 creature with no name, abilities, or creature types."
    );
    supported("Mirror Mockery");
    let mut t = TestGame::new(2);
    let fd = manifest_card(&mut t, P0, "Serra Angel");
    t.g.objects[fd.0 as usize].summoning_sick = false;
    attach_new(&mut t, P0, "Mirror Mockery", fd);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(fd, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(blank_2_2(&t, toks[0]));
    // "Exile that token at end of combat."
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!t.g.is_live(toks[0]));
    assert!(t.on_battlefield(fd));
}
