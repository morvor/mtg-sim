//! Rulings batch S25 — a token copy gets only the copiable values of what it copies
//! (CR 707.2): not its status (tapped), counters, attachments, or non-copy effects; a copy
//! of a token that isn't copying anything uses the characteristics its creator gave it
//! (CR 111.4, 707.2).

use crate::r_s01_common::{supported, tokens};
use crate::r_s06_common::activate_containing;
use crate::r_s13_common::add;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Makes the permanent tapped, gives it a +1/+1 counter, and casts Giant Growth on it.
fn tap_counter_and_pump(t: &mut TestGame, id: ObjectId) {
    t.g.tap(id);
    add(t, id, counters::PLUS1, 1);
    cast_new(t, P0, "Giant Growth", &[Entity::Object(id)]);
    t.resolve_all();
}

/// The tokens P0 controls that aren't in `before`.
fn new_tokens(t: &TestGame, before: &[ObjectId]) -> Vec<ObjectId> {
    tokens(t, P0)
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect()
}

/// P0 creates two Soldier tokens with Raise the Alarm; the first is tapped, gets a +1/+1
/// counter and Giant Growth. Returns it.
fn modified_soldier_token(t: &mut TestGame) -> ObjectId {
    cast_new(t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let soldier = tokens(t, P0)[0];
    assert_eq!(name_now(t, soldier), "Soldier Token");
    tap_counter_and_pump(t, soldier);
    assert_eq!(t.pt(soldier), (5, 5));
    soldier
}

/// The new token is an untapped 1/1 white Soldier named "Soldier Token" with no counters.
fn plain_soldier(t: &TestGame, token: ObjectId) {
    let o = t.obj_now(token);
    assert_eq!(o.chars.name, "Soldier Token");
    assert_eq!(t.pt(token), (1, 1));
    assert!(!o.tapped);
    assert_eq!(t.counters(token, counters::PLUS1), 0);
    assert!(o.chars.colors.contains(Color::White));
}

#[test]
fn a_populated_token_doesnt_copy_status_counters_or_effects() {
    cr!("707.2", "701.36a");
    ruling!(
        "Cayth, Famed Mechanist",
        "The new token doesn't copy whether the original token is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, color, and so on."
    );
    supported("Cayth, Famed Mechanist");
    // "{2}, {T}: Choose one — • Populate. • Proliferate."
    let mut t = TestGame::new(2);
    let cayth = t.battlefield(P0, "Cayth, Famed Mechanist");
    let soldier = modified_soldier_token(&mut t);
    let before = tokens(&t, P0);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    activate_containing(&mut t, P0, cayth, "Choose one").unwrap();
    t.resolve_all();
    let new = new_tokens(&t, &before);
    assert_eq!(new.len(), 1);
    plain_soldier(&t, new[0]);
}

#[test]
fn a_copy_of_a_token_uses_the_characteristics_its_creator_gave_it() {
    cr!("707.2", "111.4");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "If a copied creature is a token that isn't a copy of something else, the copy copies the original characteristics of that token as stated by the effect that created it."
    );
    supported("Kiki-Jiki, Mirror Breaker");
    // "{T}: Create a token that's a copy of target nonlegendary creature you control,
    // except it has haste."
    let mut t = TestGame::new(2);
    let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    let soldier = modified_soldier_token(&mut t);
    let before = tokens(&t, P0);
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    activate_containing(&mut t, P0, kiki, "Create a token").unwrap();
    t.resolve_all();
    let new = new_tokens(&t, &before);
    assert_eq!(new.len(), 1);
    plain_soldier(&t, new[0]);
    assert!(t.obj_now(new[0]).has_keyword(KeywordKind::Haste));
}

#[test]
fn a_token_copy_of_a_creature_copies_only_what_was_printed() {
    cr!("707.2");
    ruling!(
        "Self-Reflection",
        "The token copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras or Equipment attached to it, or any non-copy effects that have changed its types, color, power and toughness, and so on."
    );
    supported("Self-Reflection");
    supported("Holy Strength");
    // "Create a token that's a copy of target creature you control."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Holy Strength", &[Entity::Object(giant)]);
    t.resolve_all();
    tap_counter_and_pump(&mut t, giant);
    assert_eq!(t.pt(giant), (3 + 1 + 1 + 3, 3 + 2 + 1 + 3));
    cast_new(&mut t, P0, "Self-Reflection", &[Entity::Object(giant)]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let token = toks[0];
    assert_eq!(name_now(&t, token), "Hill Giant");
    assert_eq!(t.pt(token), (3, 3));
    assert!(!t.obj_now(token).tapped);
    assert_eq!(t.counters(token, counters::PLUS1), 0);
    assert!(t
        .g
        .attachments_of(Entity::Object(t.g.current(token)))
        .is_empty());
}

#[test]
fn a_token_copy_of_an_artifact_copies_only_what_was_printed() {
    cr!("707.2");
    ruling!(
        "Saheeli's Artistry",
        "The token copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Saheeli's Artistry");
    // "• Create a token that's a copy of target artifact."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    tap_counter_and_pump(&mut t, thopter);
    assert_eq!(t.pt(thopter), (4, 6));
    lands_for_cost(&mut t, P0, "Saheeli's Artistry");
    let card = t.hand(P0, "Saheeli's Artistry");
    t.cast(P0, card).modes(&[0]).target(thopter).go();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let token = toks[0];
    assert_eq!(name_now(&t, token), "Ornithopter");
    assert_eq!(t.pt(token), (0, 2));
    assert!(!t.obj_now(token).tapped);
    assert_eq!(t.counters(token, counters::PLUS1), 0);
}

#[test]
fn octomancer_copies_the_original_characteristics_of_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Octomancer",
        "The token you create copies the original characteristics of the token as stated by the effect that created that token (unless that token is copying something else; see below). It doesn't copy whether that token is tapped or untapped"
    );
    supported("Octomancer");
    // "At the beginning of each end step, create a token that's a copy of target creature
    // token that entered the battlefield this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Octomancer");
    let soldier = modified_soldier_token(&mut t);
    let before = tokens(&t, P0);
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // (Giant Growth still pumps the original.)
    assert_eq!(t.pt(soldier), (5, 5));
    let new = new_tokens(&t, &before);
    assert_eq!(new.len(), 1);
    plain_soldier(&t, new[0]);
}

#[test]
fn romana_ii_copies_only_a_token_that_entered_this_turn() {
    cr!("707.2", "111.4", "115.1");
    ruling!(
        "Romana II",
        "The token you create copies the original characteristics of the token as stated by the effect that created that token"
    );
    supported("Romana II");
    // "{1}, {T}: Create a tapped token that's a copy of target token that entered this
    // turn."
    let mut t = TestGame::new(2);
    let romana = t.battlefield(P0, "Romana II");
    // A token from an earlier turn isn't a legal target.
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let old = tokens(&t, P0);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let soldier = new_tokens(&t, &old)[0];
    tap_counter_and_pump(&mut t, soldier);
    assert_eq!(t.pt(soldier), (5, 5));
    let before = tokens(&t, P0);
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    activate_containing(&mut t, P0, romana, "Create a tapped token").unwrap();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&Entity::Object(soldier)));
    for o in &old {
        assert!(!offered[0].contains(&Entity::Object(*o)));
    }
    t.resolve_all();
    let new = new_tokens(&t, &before);
    assert_eq!(new.len(), 1);
    let token = new[0];
    let o = t.obj_now(token);
    assert_eq!(o.chars.name, "Soldier Token");
    assert_eq!(t.pt(token), (1, 1));
    assert_eq!(t.counters(token, counters::PLUS1), 0);
    // Romana II creates it tapped.
    assert!(o.tapped);
}
