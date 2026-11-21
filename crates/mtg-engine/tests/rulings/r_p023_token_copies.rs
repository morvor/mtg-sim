//! Rulings batch P023 — effects that create a token that's a copy of a creature: if the
//! copied creature is a token, the new token copies the original characteristics the
//! effect that created that token gave it (CR 707.2, 111.3); if it's copying something
//! else, the new token is whatever it copied, as modified by that copy effect's
//! exceptions (CR 707.3, 707.9b); a copied {X} is 0 (CR 107.3g, 202.3e).

use crate::r_p023_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s18_common::lands_for;
use crate::r_s26_common::{dress_up, new_tokens};
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts `p`'s `spell` (paying `cost` with basic lands) with `targets`, optional `modes`
/// and `x`, resolves everything, and returns the tokens `p` got.
fn cast_for_tokens(
    t: &mut TestGame,
    p: PlayerId,
    spell: &str,
    cost: &str,
    targets: &[Entity],
    modes: Option<&[usize]>,
    x: Option<i64>,
) -> Vec<ObjectId> {
    supported(spell);
    lands_for(t, p, cost);
    let card = t.hand(p, spell);
    let before = t.g.battlefield.clone();
    // One target per target spec, in order.
    let mut b = t.cast(p, card);
    for e in targets {
        b = b.target(*e);
    }
    if let Some(m) = modes {
        b = b.modes(m);
    }
    if let Some(x) = x {
        b = b.x(x);
    }
    b.go();
    t.resolve_all();
    new_tokens(t, p, &before)
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Casts `spell` targeting `p`'s dressed-up Wolf token; asserts one new token, a fresh copy
/// of the Wolf's original characteristics (2/2 when `pt`).
fn spell_copies_wolf(spell: &str, cost: &str, modes: Option<&[usize]>, pt: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let toks = cast_for_tokens(&mut t, P0, spell, cost, &[obj(wolf)], modes, None);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, pt);
    (t, toks[0])
}

/// Casts `spell` targeting `p`'s Clone that's copying Serra Angel; asserts one new token,
/// a Serra Angel.
fn spell_copies_cloned_angel(
    spell: &str,
    cost: &str,
    modes: Option<&[usize]>,
    pt: bool,
) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let toks = cast_for_tokens(&mut t, P0, spell, cost, &[obj(clone)], modes, None);
    assert_eq!(toks.len(), 1);
    assert_angel(&t, toks[0], pt);
    (t, toks[0])
}

fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn legendary(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.supertypes.contains(Supertype::Legendary)
}

// --- Spells: "create a token that's a copy of target creature you control" --------------

#[test]
fn self_reflection_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Self-Reflection",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    spell_copies_wolf("Self-Reflection", "{4}{U}{U}", None, true);
}

#[test]
fn cackling_counterpart_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Cackling Counterpart",
        "If the copied creature is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created the token."
    );
    spell_copies_wolf("Cackling Counterpart", "{1}{U}{U}", None, true);
}

#[test]
fn quasiduplicate_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Quasiduplicate",
        "If the copied creature is itself a token, the token created by Quasiduplicate copies the original characteristics of that token as stated by the effect that created it."
    );
    spell_copies_wolf("Quasiduplicate", "{1}{U}{U}", None, true);
}

#[test]
fn fated_infatuation_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Fated Infatuation",
        "If the copied creature is a token, the token created by Fated Infatuation copies the original characteristics of that token as stated by the effect that put the token onto the battlefield."
    );
    spell_copies_wolf("Fated Infatuation", "{U}{U}{U}", None, true);
}

#[test]
fn rally_the_galadhrim_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Rally the Galadhrim",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token."
    );
    spell_copies_wolf("Rally the Galadhrim", "{2}{G}{U}", None, true);
}

#[test]
fn quantum_misalignment_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Quantum Misalignment",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    spell_copies_wolf("Quantum Misalignment", "{4}{U}", None, true);
}

#[test]
fn replicate_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "709.3");
    ruling!(
        "Repudiate // Replicate",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let toks = cast_replicate(&mut t, wolf);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
}

#[test]
fn replicate_copies_what_a_clone_is_copying() {
    cr!("707.3", "709.3");
    ruling!(
        "Repudiate // Replicate",
        "If the copied creature is copying something else (for example, if the copied creature is a Mirror Image), then the token enters the battlefield as whatever that creature copied."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let toks = cast_replicate(&mut t, clone);
    assert_eq!(toks.len(), 1);
    assert_angel(&t, toks[0], true);
}

/// Casts Replicate (the right half of Repudiate // Replicate) targeting `what`.
fn cast_replicate(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    supported("Repudiate // Replicate");
    lands_for(t, P0, "{1}{G}{U}");
    let card = t.hand(P0, "Repudiate // Replicate");
    let before = t.g.battlefield.clone();
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(what)
        .go();
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn supplant_form_copies_a_returned_tokens_original_characteristics() {
    cr!("707.2", "111.3", "608.2h");
    ruling!(
        "Supplant Form",
        "If the copied creature is a token, the token created by Supplant Form copies the original characteristics of that token as stated by the effect that put it onto the battlefield."
    );
    let (t, _) = spell_copies_wolf("Supplant Form", "{4}{U}{U}", None, true);
    // The original token left the battlefield (and ceased to exist).
    assert_eq!(t.named_on_battlefield("Wolf").len(), 1);
}

#[test]
fn sublime_epiphany_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "700.2");
    ruling!(
        "Sublime Epiphany",
        "If the copied creature is itself a token, the token created by Sublime Epiphany copies the original characteristics of that token as stated by the effect that created it."
    );
    spell_copies_wolf("Sublime Epiphany", "{4}{U}{U}", Some(&[3]), true);
}

#[test]
fn mirage_mockery_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "700.2");
    ruling!(
        "Mirage Mockery",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created that token."
    );
    spell_copies_wolf("Mirage Mockery", "{2}{U}", Some(&[1]), true);
}

#[test]
fn clone_legion_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Clone Legion",
        "If the copied creature is a token, the token created by Clone Legion copies the original characteristics of that token as stated by the effect that put the token onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Clone Legion",
        "{7}{U}{U}",
        &[Entity::Player(P1)],
        None,
        None,
    );
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
    assert!(t.on_battlefield(wolf));
}

#[test]
fn croaking_counterpart_copies_a_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Croaking Counterpart",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created that token, with the exceptions noted above."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let toks = cast_for_tokens(&mut t, P0, "Croaking Counterpart", "{1}{G}{U}", &[obj(wolf)], None, None);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Wolf");
    assert!(o.is_token());
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert!(has_subtype(&t, toks[0], "Frog") && !has_subtype(&t, toks[0], "Wolf"));
    assert_eq!(t.pt(toks[0]), (1, 1));
}

#[test]
fn electroduplicate_copies_a_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9a");
    ruling!(
        "Electroduplicate",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token, with the listed exceptions."
    );
    let (t, tok) = spell_copies_wolf("Electroduplicate", "{2}{R}", None, true);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
}

#[test]
fn kindle_the_inner_flame_copies_a_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9a");
    ruling!(
        "Kindle the Inner Flame",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token, with the listed exceptions."
    );
    let (t, tok) = spell_copies_wolf("Kindle the Inner Flame", "{3}{R}", None, true);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
}

#[test]
fn ember_island_production_copies_a_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Ember Island Production",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token, with the listed exceptions."
    );
    let (t, tok) = spell_copies_wolf("Ember Island Production", "{3}{U}{U}", Some(&[0]), false);
    assert_eq!(t.pt(tok), (4, 4));
    assert!(has_subtype(&t, tok, "Hero"));
}

#[test]
fn ember_island_production_copies_what_a_clone_is_copying_with_the_exceptions() {
    cr!("707.3", "707.9b");
    ruling!(
        "Ember Island Production",
        "If the copied creature is copying something else (for example, if the copied creature is an Evil Twin), then the token enters as whatever that creature copied, with the listed exceptions."
    );
    let (t, tok) = spell_copies_cloned_angel("Ember Island Production", "{3}{U}{U}", Some(&[0]), true);
    assert!(has_subtype(&t, tok, "Hero") && has_subtype(&t, tok, "Angel"));
}

#[test]
fn hate_mirage_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Hate Mirage",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let toks = cast_for_tokens(&mut t, P0, "Hate Mirage", "{3}{R}", &[obj(wolf)], None, None);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn hate_mirage_copies_what_a_clone_is_copying() {
    cr!("707.3");
    ruling!(
        "Hate Mirage",
        "If the copied creature is copying something else (for example, if Volrath, the Shapestealer is having its own shape stolen by Hate Mirage), then the token enters the battlefield as whatever that creature copied."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let toks = cast_for_tokens(&mut t, P0, "Hate Mirage", "{3}{R}", &[obj(clone)], None, None);
    assert_eq!(toks.len(), 1);
    assert_angel(&t, toks[0], true);
}

#[test]
fn tempt_with_reflections_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Tempt with Reflections",
        "If the copied creature is a token, the tokens created by Tempt with Reflections copy the original characteristics of that token as stated by the effect that put the token onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    t.answer_yes(P1, true);
    let before = t.g.battlefield.clone();
    let toks = cast_for_tokens(&mut t, P0, "Tempt with Reflections", "{3}{U}", &[obj(wolf)], None, None);
    assert_eq!(toks.len(), 2);
    let theirs = new_tokens(&t, P1, &before);
    assert_eq!(theirs.len(), 1);
    for tok in toks.into_iter().chain(theirs) {
        assert_wolf(&t, tok, true, true);
    }
}

#[test]
fn tempt_with_reflections_copies_what_a_clone_is_copying() {
    cr!("707.3");
    ruling!(
        "Tempt with Reflections",
        "If the copied creature is copying something else (for example, if the copied creature is a Clone), then the tokens enter the battlefield as whatever that creature copied."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    t.answer_yes(P1, true);
    let before = t.g.battlefield.clone();
    let toks = cast_for_tokens(&mut t, P0, "Tempt with Reflections", "{3}{U}", &[obj(clone)], None, None);
    assert_eq!(toks.len(), 2);
    let theirs = new_tokens(&t, P1, &before);
    assert_eq!(theirs.len(), 1);
    for tok in toks.into_iter().chain(theirs) {
        assert_angel(&t, tok, true);
    }
}

#[test]
fn aggressive_biomancy_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "707.9a");
    ruling!(
        "Aggressive Biomancy",
        "If the copied creature is itself a token, the tokens created by Aggressive Biomancy copy the original characteristics of that token as stated by the effect that created it, with the listed exception."
    );
    supported("Aggressive Biomancy");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Aggressive Biomancy",
        "{4}{G}{U}",
        &[obj(wolf)],
        None,
        Some(2),
    );
    assert_eq!(toks.len(), 2);
    for tok in toks {
        assert_wolf(&t, tok, true, true);
        assert!(t.obj_now(tok).chars.abilities.iter().any(|a| a.text.contains("fights")));
    }
}

#[test]
fn aggressive_biomancy_copies_what_a_clone_is_copying() {
    cr!("707.3", "707.9a");
    ruling!(
        "Aggressive Biomancy",
        "If the copied creature is copying something else, then the tokens enter the battlefield as whatever that creature copied, with the listed exception."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Aggressive Biomancy",
        "{2}{G}{U}",
        &[obj(clone)],
        None,
        Some(1),
    );
    assert_eq!(toks.len(), 1);
    assert_angel(&t, toks[0], true);
    assert!(t.obj_now(toks[0]).chars.abilities.iter().any(|a| a.text.contains("fights")));
}

#[test]
fn here_comes_a_new_hero_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Here Comes a New Hero!",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Here Comes a New Hero!",
        "{2}{U}",
        &[Entity::Player(P0), obj(wolf)],
        None,
        Some(0),
    );
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
}

#[test]
fn here_comes_a_new_hero_treats_x_in_the_copied_cost_as_zero() {
    cr!("107.3g", "202.3e");
    ruling!(
        "Here Comes a New Hero!",
        "If the copied creature has {X} in its mana cost, X is 0. This is also true when calculating the mana value of a creature on the battlefield in order to determine whether or not the creature is a legal target for this spell."
    );
    let mut t = TestGame::new(2);
    // Benevolent Hydra ({X}{G}{G}) cast with X = 3: its mana value on the battlefield is
    // 2, so it's a legal target for X = 2.
    let hydra = hydra_cast_with_x3(&mut t, P0);
    assert_eq!(mv_now(&mut t, hydra), 2);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Here Comes a New Hero!",
        "{2}{U}{U}{U}",
        &[Entity::Player(P1), obj(hydra)],
        None,
        Some(2),
    );
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Benevolent Hydra");
    assert_eq!(mv_now(&mut t, toks[0]), 2);
    assert_eq!(t.counters(toks[0], counters::PLUS1), 0);
    assert_eq!(t.pt(toks[0]), (1, 1));
}

// --- Activated abilities ------------------------------------------------------------------

#[test]
fn orthion_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Orthion, Hero of Lavabrink",
        "If the copied creature is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created that token."
    );
    supported("Orthion, Hero of Lavabrink");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let orthion = t.battlefield(P0, "Orthion, Hero of Lavabrink");
    let wolf = dressed_wolf(&mut t, P0);
    lands_for(&mut t, P0, "{1}{R}");
    let before = t.g.battlefield.clone();
    t.activate(P0, orthion, 0, &[obj(wolf)]).expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn the_fire_crystal_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "The Fire Crystal",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let toks = fire_crystal_copy(&mut t, wolf);
    assert_wolf(&t, toks[0], true, true);
}

#[test]
fn the_fire_crystal_treats_x_in_the_copied_cost_as_zero() {
    cr!("107.3g", "107.3m", "202.3e", "707.2");
    ruling!(
        "The Fire Crystal",
        "If the copied creature has {X} in its mana cost, X is 0."
    );
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let toks = fire_crystal_copy(&mut t, hydra);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Benevolent Hydra");
    assert_eq!(mv_now(&mut t, toks[0]), 2);
    assert_eq!(t.counters(toks[0], counters::PLUS1), 0);
    assert_eq!(t.pt(toks[0]), (1, 1));
}

/// Activates P0's The Fire Crystal targeting `what`; returns the one new token.
fn fire_crystal_copy(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    supported("The Fire Crystal");
    let crystal = t.battlefield(P0, "The Fire Crystal");
    lands_for(t, P0, "{4}{R}{R}");
    t.answer_targets(P0, &[obj(what)]);
    let before = t.g.battlefield.clone();
    activate_containing(t, P0, crystal, "Create a token").expect("activate");
    t.resolve_all();
    let toks = new_tokens(t, P0, &before);
    assert_eq!(toks.len(), 1);
    toks
}

#[test]
fn tempestra_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Tempestra, Dame of Games",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let tok = tempestra_copy(&mut t, wolf);
    assert_wolf(&t, tok, true, true);
}

#[test]
fn tempestra_copies_what_a_clone_is_copying() {
    cr!("707.3", "707.9b");
    ruling!(
        "Tempestra, Dame of Games",
        "If the copied creature is copying something else, then the token enters as whatever that creature copied (with the listed exception)."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let tok = tempestra_copy(&mut t, clone);
    assert_angel(&t, tok, true);
}

/// Activates P0's Tempestra (sacrificing an Ornithopter) targeting `what`.
fn tempestra_copy(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Tempestra, Dame of Games");
    let tempestra = t.battlefield(P0, "Tempestra, Dame of Games");
    let bird = t.battlefield(P0, "Ornithopter");
    lands_for(t, P0, "{2}{R}");
    t.answer_choose(P0, &[obj(bird)]);
    let before = t.g.battlefield.clone();
    t.activate(P0, tempestra, 0, &[obj(what)]).expect("activate");
    t.resolve_all();
    assert!(!t.on_battlefield(bird));
    let toks = new_tokens(t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(has_kw(t, toks[0], KeywordKind::Haste));
    assert!(!legendary(t, toks[0]));
    toks[0]
}

#[test]
fn the_jolly_balloon_man_copies_a_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "The Jolly Balloon Man",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token, with the stated exceptions."
    );
    supported("The Jolly Balloon Man");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let man = t.battlefield(P0, "The Jolly Balloon Man");
    let wolf = dressed_wolf(&mut t, P0);
    lands_for(&mut t, P0, "{1}");
    let before = t.g.battlefield.clone();
    t.activate(P0, man, 0, &[obj(wolf)]).expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Wolf");
    assert!(o.chars.colors.contains(Color::Green) && o.chars.colors.contains(Color::Red));
    assert!(!o.chars.colors.contains(Color::Blue));
    assert!(has_subtype(&t, toks[0], "Wolf") && has_subtype(&t, toks[0], "Balloon"));
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(has_kw(&t, toks[0], KeywordKind::Flying));
}

// --- Triggered abilities -------------------------------------------------------------------

#[test]
fn helm_of_the_host_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Helm of the Host",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token."
    );
    supported("Helm of the Host");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    attach_new(&mut t, P0, "Helm of the Host", wolf);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn penumbra_umbra_copies_a_tokens_original_characteristics_with_the_exception() {
    cr!("707.2", "111.3", "707.9b", "603.10a");
    ruling!(
        "Penumbra Umbra",
        "If the copied creature is a token, the new token that’s created copies the original characteristics of that token, with the exception noted above."
    );
    supported("Penumbra Umbra");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let umbra = attach_new(&mut t, P0, "Penumbra Umbra", wolf);
    let before = t.g.battlefield.clone();
    destroy(&mut t, umbra);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Wolf");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn penumbra_umbra_copies_what_a_clone_is_copying_with_the_exception() {
    cr!("707.3", "707.9b", "603.10a");
    ruling!(
        "Penumbra Umbra",
        "If the copied creature is copying something else, then the token enters the battlefield as whatever that creature copied, with the exception noted above."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let umbra = attach_new(&mut t, P0, "Penumbra Umbra", clone);
    let before = t.g.battlefield.clone();
    destroy(&mut t, umbra);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Serra Angel");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert_eq!(t.pt(toks[0]), (4, 4));
}

/// `p`'s legendary 2/2 green Wolf token named "Alpha Wolf".
fn legendary_wolf(t: &mut TestGame, p: PlayerId) -> ObjectId {
    token_of(
        t,
        p,
        TokenSpec {
            name: "Alpha Wolf".into(),
            colors: ColorSet::single(Color::Green),
            supertypes: vec![Supertype::Legendary],
            card_types: vec![CardType::Creature],
            subtypes: vec!["Wolf".into()],
            power: Some(2),
            toughness: Some(2),
            abilities: vec![],
            scryfall_name: None,
            pt_values: None,
        },
    )
}

#[test]
fn ratadrabik_copies_a_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9b", "603.10a");
    ruling!(
        "Ratadrabik of Urborg",
        "If the copied creature is a token, the new token that’s created copies the original characteristics of that token as stated by the effect that created that token, with the exceptions noted above."
    );
    supported("Ratadrabik of Urborg");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ratadrabik of Urborg");
    let alpha = legendary_wolf(&mut t, P0);
    dress_up(&mut t, alpha);
    let before = t.g.battlefield.clone();
    destroy(&mut t, alpha);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Alpha Wolf");
    assert!(!legendary(&t, toks[0]));
    assert!(o.chars.colors.contains(Color::Green) && o.chars.colors.contains(Color::Black));
    assert!(!o.chars.colors.contains(Color::Blue));
    assert!(has_subtype(&t, toks[0], "Wolf") && has_subtype(&t, toks[0], "Zombie"));
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn nightmare_shepherd_treats_x_in_the_copied_cost_as_zero() {
    cr!("107.3g", "107.3m", "202.3e", "707.2");
    ruling!(
        "Nightmare Shepherd",
        "If the copied creature had {X} in its mana cost, X is considered to be 0."
    );
    supported("Nightmare Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightmare Shepherd");
    let hydra = hydra_cast_with_x3(&mut t, P0);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    destroy(&mut t, hydra);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Benevolent Hydra");
    assert_eq!(mv_now(&mut t, toks[0]), 2);
    assert_eq!(t.counters(toks[0], counters::PLUS1), 0);
    assert_eq!(t.pt(toks[0]), (1, 1));
}

#[test]
fn felhide_spiritbinder_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Felhide Spiritbinder",
        "If the copied creature is a token, the token created by Felhide Spiritbinder copies the original characteristics of that token as stated by the effect that put the token onto the battlefield."
    );
    supported("Felhide Spiritbinder");
    let mut t = TestGame::new(2);
    let binder = t.battlefield(P0, "Felhide Spiritbinder");
    let wolf = dressed_wolf(&mut t, P1);
    lands_for(&mut t, P0, "{1}{R}");
    t.g.tap(binder);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wolf)]);
    let before = t.g.battlefield.clone();
    t.g.untap(binder);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
    assert!(t.obj_now(toks[0]).is(CardType::Enchantment));
}

#[test]
fn flamerush_rider_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "508.4");
    ruling!(
        "Flamerush Rider",
        "If the copied creature is a token, the token created by Flamerush Rider copies the original characteristics of that token as stated by the effect that put it onto the battlefield."
    );
    supported("Flamerush Rider");
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Flamerush Rider");
    let wolf = wolf_token(&mut t, P0);
    t.g.add_counters(Entity::Object(wolf), counters::PLUS1, 2, None);
    t.answer_targets(P0, &[obj(wolf)]);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(rider, Entity::Player(P1)), (wolf, Entity::Player(P1))], &[]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(is_wolf(&t, toks[0]) && t.obj_now(toks[0]).is_token());
    assert!(t.obj_now(toks[0]).tapped);
    // Rider 3 + Wolf 4 + the 2/2 copy.
    assert_eq!(t.life(P1), 20 - 3 - 4 - 2);
}

#[test]
fn shaun_copies_a_legendary_tokens_original_characteristics_with_the_exceptions() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Shaun, Father of Synths",
        "If the copied creature is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created that token, with the noted exceptions."
    );
    supported("Shaun, Father of Synths");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shaun, Father of Synths");
    let alpha = legendary_wolf(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(alpha)]);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(alpha, Entity::Player(P1))], &[]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Alpha Wolf");
    assert!(!legendary(&t, toks[0]));
    assert!(o.is(CardType::Artifact) && has_subtype(&t, toks[0], "Synth"));
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn shaun_copies_what_a_clone_is_copying_with_the_exceptions() {
    cr!("707.3", "707.9b");
    ruling!(
        "Shaun, Father of Synths",
        "If the copied creature is copying something else, then the token enters the battlefield as whatever that creature copied, with the noted exceptions."
    );
    supported("Shaun, Father of Synths");
    supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shaun, Father of Synths");
    let thalia = t.battlefield(P1, "Thalia, Guardian of Thraben");
    let clone = clone_of(&mut t, P0, thalia);
    unsick(&mut t, clone);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(clone)]);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(clone, Entity::Player(P1))], &[]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Thalia, Guardian of Thraben");
    assert!(!legendary(&t, toks[0]));
    assert!(o.is(CardType::Artifact) && has_subtype(&t, toks[0], "Synth"));
    assert!(has_kw(&t, toks[0], KeywordKind::FirstStrike));
}

#[test]
fn grub_copies_a_blighted_tokens_original_characteristics() {
    cr!("707.2", "111.3", "701.68a");
    ruling!(
        "Grub, Storied Matriarch // Grub, Notorious Auntie",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token, with the listed exception."
    );
    let mut t = TestGame::new(2);
    let grub = auntie(&mut t);
    let wolf = wolf_token(&mut t, P0);
    let tok = grub_copy(&mut t, grub, wolf);
    // The Wolf has a -1/-1 counter; the copy doesn't.
    assert_wolf(&t, tok, true, true);
    assert_eq!(t.pt(wolf), (1, 1));
}

#[test]
fn grub_copies_what_a_blighted_clone_is_copying() {
    cr!("707.3", "701.68a");
    ruling!(
        "Grub, Storied Matriarch // Grub, Notorious Auntie",
        "If the copied creature is copying something else, then the token enters as whatever that creature copied, with the listed exception."
    );
    let mut t = TestGame::new(2);
    let grub = auntie(&mut t);
    let clone = cloned_angel(&mut t, P0);
    let tok = grub_copy(&mut t, grub, clone);
    assert_angel(&t, tok, true);
}

/// P0's Grub, transformed to Grub, Notorious Auntie.
fn auntie(t: &mut TestGame) -> ObjectId {
    supported("Grub, Storied Matriarch");
    let grub = t.battlefield(P0, "Grub, Storied Matriarch");
    mtg_engine::dfc::transform(&mut t.g, grub);
    t.g.recompute();
    assert_eq!(t.obj(grub).chars.name.as_str(), "Grub, Notorious Auntie");
    grub
}

/// Grub attacks and blights `what`; returns the token copy.
fn grub_copy(t: &mut TestGame, grub: ObjectId, what: ObjectId) -> ObjectId {
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(what)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(grub, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(what, "-1/-1"), 1);
    let toks = new_tokens(t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(t.obj_now(toks[0]).tapped);
    assert!(t.obj_now(toks[0]).chars.abilities.iter().any(|a| a.text.contains("sacrifice ~")));
    toks[0]
}

#[test]
fn mirror_room_copies_a_tokens_original_characteristics_with_the_exception() {
    cr!("707.2", "111.3", "707.9b", "709.5");
    ruling!(
        "Mirror Room // Fractured Realm",
        "If the copied creature is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token, with the stated exception."
    );
    supported("Mirror Room // Fractured Realm");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    lands_for(&mut t, P0, "{2}{U}");
    let card = t.hand(P0, "Mirror Room // Fractured Realm");
    let before = t.g.battlefield.clone();
    t.cast(P0, card)
        .method(CastMethod::Half(0))
        .go();
    t.answer_targets(P0, &[obj(wolf)]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_wolf(&t, toks[0], true, true);
    assert!(has_subtype(&t, toks[0], "Reflection"));
}
