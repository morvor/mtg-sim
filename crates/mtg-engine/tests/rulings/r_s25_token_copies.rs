//! Rulings batch S25 — a token that's a copy of a permanent that is itself copying
//! something enters as whatever that permanent copied: the copiable values of a copy are
//! the values it copied (CR 707.3, 707.2), with the new effect's exceptions (CR 707.9).

use crate::r_s01_common::{supported, tokens};
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::token_copy;
use crate::r_s25_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 controls Serra Angel (a 4/4 flying Angel); P0's Clone enters as a copy of it.
/// Returns the Clone.
fn clone_of_serra_angel(t: &mut TestGame) -> ObjectId {
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[Entity::Object(angel)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(name_now(t, clone), "Serra Angel");
    clone
}

/// P0's only new token after `copy` (which makes a token copy of P0's Clone of Serra
/// Angel) is a Serra Angel with flying, not a Shapeshifter (what Clone's card has).
/// Returns (game, token).
fn token_copy_of_a_clone(copy: impl FnOnce(&mut TestGame, ObjectId)) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let clone = clone_of_serra_angel(&mut t);
    copy(&mut t, clone);
    t.g.recompute();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1, "one token expected");
    let token = toks[0];
    assert_eq!(name_now(&t, token), "Serra Angel");
    assert!(t.obj_now(token).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(token).chars.has_subtype("Shapeshifter"));
    (t, token)
}

/// P0 casts the real spell `name` targeting the Clone, and it resolves.
fn cast_at_the_clone(name: &str) -> (TestGame, ObjectId) {
    supported(name);
    token_copy_of_a_clone(|t, clone| {
        cast_new(t, P0, name, &[Entity::Object(clone)]);
        t.resolve_all();
    })
}

#[test]
fn cackling_counterpart_copies_what_a_clone_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Cackling Counterpart",
        "If the copied creature is copying something else, then the token enters the battlefield as whatever that creature copied."
    );
    let (t, token) = cast_at_the_clone("Cackling Counterpart");
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn relm_s_sketching_copies_what_a_clone_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Relm's Sketching",
        "If the copied permanent is copying something else, then the token enters as whatever that permanent copied."
    );
    let (t, token) = cast_at_the_clone("Relm's Sketching");
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn self_reflection_copies_what_a_clone_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Self-Reflection",
        "If the copied creature is copying something else, then the token enters as whatever that creature copied."
    );
    let (t, token) = cast_at_the_clone("Self-Reflection");
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn quasiduplicate_copies_what_a_clone_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Quasiduplicate",
        "If the copied creature is copying something else, the token enters the battlefield as whatever that creature is copying."
    );
    let (t, token) = cast_at_the_clone("Quasiduplicate");
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn fated_infatuation_copies_what_a_clone_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Fated Infatuation",
        "If the copied creature is copying something else (for example, if the copied creature is a Clone), then the token enters the battlefield as whatever that creature copied."
    );
    let (t, token) = cast_at_the_clone("Fated Infatuation");
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn quantum_misalignment_copies_what_a_clone_copied() {
    cr!("707.3", "707.2", "707.9b");
    ruling!(
        "Quantum Misalignment",
        "If the copied creature is copying something else (for example, if the copied creature is an Evil Twin), then the token enters the battlefield as whatever that creature copied."
    );
    let (t, token) = cast_at_the_clone("Quantum Misalignment");
    assert_eq!(t.pt(token), (4, 4));
    assert!(!legendary(&t, token));
}

#[test]
fn croaking_counterpart_copies_what_a_clone_copied_with_its_exceptions() {
    cr!("707.3", "707.9b");
    ruling!(
        "Croaking Counterpart",
        "If the copied creature is copying something else, then the token enters the battlefield as whatever that creature copied, with the exceptions noted above."
    );
    // "Create a token that's a copy of target non-Frog creature, except it's a 1/1 green
    // Frog."
    let (t, token) = cast_at_the_clone("Croaking Counterpart");
    assert_eq!(t.pt(token), (1, 1));
    let o = t.obj_now(token);
    // The exceptions replace the copied creature types and colors.
    assert!(o.chars.has_subtype("Frog"));
    assert!(!o.chars.has_subtype("Angel"));
    assert!(o.chars.colors.contains(Color::Green));
    assert!(!o.chars.colors.contains(Color::White));
}

#[test]
fn electroduplicate_copies_what_a_clone_copied_with_its_exceptions() {
    cr!("707.3", "707.9a");
    ruling!(
        "Electroduplicate",
        "If the copied creature is copying something else, then the token enters as whatever that creature copied, with the listed exceptions."
    );
    // "Create a token that's a copy of target creature you control, except it has haste and
    // 'At the beginning of the end step, sacrifice this token.'"
    let (t, token) = cast_at_the_clone("Electroduplicate");
    assert_eq!(t.pt(token), (4, 4));
    assert!(t.obj_now(token).has_keyword(KeywordKind::Haste));
}

#[test]
fn three_steps_ahead_copies_what_a_clone_copied() {
    cr!("707.3", "707.2", "702.172a");
    ruling!(
        "Three Steps Ahead",
        "If the copied permanent is copying something else, then the token enters the battlefield as whatever that permanent copied."
    );
    supported("Three Steps Ahead");
    // Spree: "+ {3} — Create a token that's a copy of target artifact or creature you
    // control."
    let (t, token) = token_copy_of_a_clone(|t, clone| {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 3);
        let card = t.hand(P0, "Three Steps Ahead");
        t.cast(P0, card).modes(&[1]).target(clone).go();
        t.resolve_all();
    });
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn supplant_form_copies_what_a_returned_clone_copied() {
    cr!("707.3", "707.2", "608.2h");
    ruling!(
        "Supplant Form",
        "If the copied creature was copying something else, the token enters the battlefield as whatever that creature was copying."
    );
    // "Return target creature to its owner's hand. You create a token that's a copy of that
    // creature."
    let (t, token) = cast_at_the_clone("Supplant Form");
    assert_eq!(t.pt(token), (4, 4));
    assert!(t.in_hand(P0, "Clone"));
}

#[test]
fn nightmare_shepherd_copies_what_a_dead_clone_copied() {
    cr!("707.3", "707.9b", "608.2h");
    ruling!(
        "Nightmare Shepherd",
        "If the copied creature was copying something else, then the token enters the battlefield as whatever that creature copied."
    );
    supported("Nightmare Shepherd");
    // "Whenever another nontoken creature you control dies, you may exile it. If you do,
    // create a token that's a copy of that creature, except it's 1/1 and it's a Nightmare
    // in addition to its other types."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightmare Shepherd");
    let clone = clone_of_serra_angel(&mut t);
    t.answer_yes(P0, true);
    crate::r_s02_common::destroy(&mut t, clone);
    t.resolve_all();
    assert!(t.in_exile("Clone"));
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let token = toks[0];
    assert_eq!(name_now(&t, token), "Serra Angel");
    assert_eq!(t.pt(token), (1, 1));
    assert!(t.obj_now(token).chars.has_subtype("Nightmare"));
    assert!(t.obj_now(token).chars.has_subtype("Angel"));
    assert!(t.obj_now(token).has_keyword(KeywordKind::Flying));
}

#[test]
fn kiki_jiki_copies_the_copiable_values_of_a_clone() {
    cr!("707.3", "707.2", "707.9a");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "If a copied creature is copying something else, the token you create will use the copiable values of the target creature. In most cases, it will just be a copy of whatever that creature is copying."
    );
    supported("Kiki-Jiki, Mirror Breaker");
    // "{T}: Create a token that's a copy of target nonlegendary creature you control,
    // except it has haste. Sacrifice it at the beginning of the next end step."
    let (t, token) = token_copy_of_a_clone(|t, clone| {
        let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
        t.answer_targets(P0, &[Entity::Object(clone)]);
        activate_containing(t, P0, kiki, "Create a token").unwrap();
        t.resolve_all();
    });
    assert_eq!(t.pt(token), (4, 4));
    assert!(t.obj_now(token).has_keyword(KeywordKind::Haste));
}

#[test]
fn saheeli_s_artistry_copies_what_a_clone_copied() {
    cr!("707.3", "707.9b");
    ruling!(
        "Saheeli's Artistry",
        "If the copied permanent is copying something else (for example, if the copied permanent is an Altered Ego), then the token enters the battlefield as whatever that permanent copied."
    );
    supported("Saheeli's Artistry");
    // "• Create a token that's a copy of target creature, except it's an artifact in
    // addition to its other types."
    let (t, token) = token_copy_of_a_clone(|t, clone| {
        lands_for_cost(t, P0, "Saheeli's Artistry");
        let card = t.hand(P0, "Saheeli's Artistry");
        t.cast(P0, card).modes(&[1]).target(clone).go();
        t.resolve_all();
    });
    assert_eq!(t.pt(token), (4, 4));
    assert!(t.obj_now(token).is(CardType::Artifact));
}

#[test]
fn cogwork_assembler_copies_what_a_sculpting_steel_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Cogwork Assembler",
        "If the copied artifact is copying something else (for example, if the copied artifact is a Sculpting Steel), then the token enters the battlefield as whatever that artifact copied."
    );
    supported("Cogwork Assembler");
    supported("Sculpting Steel");
    // "{7}: Create a token that's a copy of target artifact. That token gains haste. Exile
    // it at the beginning of the next end step." Sculpting Steel: "You may have this
    // artifact enter as a copy of any artifact on the battlefield."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.answer_yes(P0, true);
    let steel = t.enter(P0, "Sculpting Steel");
    t.settle();
    assert_eq!(name_now(&t, steel), "Ornithopter");
    let assembler = t.battlefield(P0, "Cogwork Assembler");
    t.lands(P0, "Wastes", 7);
    t.answer_targets(P0, &[Entity::Object(steel)]);
    activate_containing(&mut t, P0, assembler, "Create a token").unwrap();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_now(&t, toks[0]), "Ornithopter");
    assert_eq!(t.pt(toks[0]), (0, 2));
    assert!(t.obj_now(toks[0]).has_keyword(KeywordKind::Flying));
}

#[test]
fn worldwalker_helm_copies_what_a_token_copied() {
    cr!("707.3", "707.2", "111.4");
    ruling!(
        "Worldwalker Helm",
        "If the copied token is copying something else, then the new token enters the battlefield as whatever that token copied."
    );
    supported("Worldwalker Helm");
    // "{1}{U}, {T}: Create a token that's a copy of target artifact token you control."
    // (Its other ability adds a Map token whenever artifact tokens are created.)
    let mut t = TestGame::new(2);
    let helm = t.battlefield(P0, "Worldwalker Helm");
    let thopter = t.battlefield(P0, "Ornithopter");
    let first = token_copy(&mut t, P0, thopter);
    let first = *first
        .iter()
        .find(|id| name_now(&t, **id) == "Ornithopter")
        .expect("a token copy of Ornithopter");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(first)]);
    activate_containing(&mut t, P0, helm, "Create a token").unwrap();
    t.resolve_all();
    let thopters: Vec<ObjectId> = tokens(&t, P0)
        .into_iter()
        .filter(|id| name_now(&t, *id) == "Ornithopter")
        .collect();
    assert_eq!(thopters.len(), 2);
    for id in thopters {
        assert_eq!(t.pt(id), (0, 2));
        assert!(t.obj_now(id).has_keyword(KeywordKind::Flying));
    }
}
