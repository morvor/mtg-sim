//! Rulings batch S26 — what a token copy copies: only the copiable values of the original
//! (CR 707.2), not whether it's tapped, its counters, what's attached to it, or non-copy
//! effects; copies of tokens copy the original characteristics the creating effect gave
//! them (CR 111.4, 707.2).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::activate_named;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s24_common::choose_creature_type;
use crate::r_s26_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn molten_echoes_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.2a");
    ruling!(
        "Molten Echoes",
        "The token copies exactly what is printed on the creature and nothing else"
    );
    supported("Molten Echoes");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Molten Echoes");
    t.settle();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    // With the trigger on the stack, the Bears are tapped, get counters, a pump and a
    // color change.
    dress_up(&mut t, bears);
    assert_eq!(t.pt(bears), (7, 7));
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert_eq!(t.obj_now(tok).chars.name, "Grizzly Bears");
    assert!(fresh(&t, tok));
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::single(Color::Green));
    // "That token gains haste" is an effect on the token, not part of the copy.
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
}

/// Sol Ring with Ensoul Artifact attached (a non-copy effect that makes it a 5/5
/// creature), tapped, with two +1/+1 counters, +3/+3 and green-blue until end of turn.
fn dressed_sol_ring(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let ring = t.battlefield(p, "Sol Ring");
    attach_new(t, p, "Ensoul Artifact", ring);
    dress_up(t, ring);
    assert!(t.obj_now(ring).is(CardType::Creature));
    assert!(t.pt(ring).1 == 10);
    ring
}

/// The token is an untapped Sol Ring with nothing attached, no counters, colorless, and
/// not a creature.
fn assert_plain_sol_ring(t: &TestGame, tok: ObjectId) {
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, "Sol Ring");
    assert!(o.is_token());
    assert!(fresh(t, tok));
    assert!(o.is(CardType::Artifact));
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless());
}

#[test]
fn echo_storm_token_copies_only_the_printed_artifact() {
    cr!("707.2", "111.4");
    ruling!(
        "Echo Storm",
        "The token copies exactly what was printed on the original artifact and nothing else"
    );
    supported("Echo Storm");
    let mut t = TestGame::new(2);
    let ring = dressed_sol_ring(&mut t, P0);
    t.lands(P0, "Island", 5);
    let storm = t.hand(P0, "Echo Storm");
    let before = t.g.battlefield.clone();
    t.cast(P0, storm).target(ring).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_plain_sol_ring(&t, toks[0]);
}

#[test]
fn chrome_dome_token_copies_only_the_printed_artifact() {
    cr!("707.2", "111.4");
    ruling!(
        "Chrome Dome",
        "The token copies exactly what was printed on the original artifact (unless that artifact is copying something else or is a token; see below). It doesn't copy whether that artifact is tapped or untapped"
    );
    supported("Chrome Dome");
    let mut t = TestGame::new(2);
    let dome = t.battlefield(P0, "Chrome Dome");
    let ring = dressed_sol_ring(&mut t, P0);
    t.lands(P0, "Wastes", 5);
    let before = t.g.battlefield.clone();
    t.activate(P0, dome, 0, &[Entity::Object(ring)])
        .expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_plain_sol_ring(&t, toks[0]);
}

#[test]
fn fated_infatuation_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.2a");
    ruling!(
        "Fated Infatuation",
        "The token copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    supported("Fated Infatuation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    dress_up(&mut t, bears);
    assert_eq!(t.pt(bears), (8, 9));
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Fated Infatuation");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(fresh(&t, toks[0]));
    assert_eq!(t.pt(toks[0]), (2, 2));
    assert_eq!(t.obj_now(toks[0]).chars.colors, ColorSet::single(Color::Green));
}

#[test]
fn relms_sketching_token_copies_only_the_printed_land() {
    cr!("707.2", "111.4");
    ruling!(
        "Relm's Sketching",
        "The token copies exactly what was printed on the original permanent (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped"
    );
    supported("Relm's Sketching");
    let mut t = TestGame::new(2);
    // Mutavault animated by its own ability (a non-copy effect that changes its types
    // and power and toughness), then tapped, with counters.
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Island", 5);
    activate_containing(&mut t, P0, vault, "becomes").expect("animate");
    t.resolve();
    assert!(t.obj_now(vault).is(CardType::Creature));
    t.g.tap(vault);
    t.g.add_counters(Entity::Object(vault), counters::PLUS1, 1, None);
    t.g.recompute();
    let spell = t.hand(P0, "Relm's Sketching");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).target(vault).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Mutavault");
    assert!(tok.is(CardType::Land));
    assert!(!tok.is(CardType::Creature));
    assert!(fresh(&t, toks[0]));
}

#[test]
fn helm_of_the_host_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Helm of the Host",
        "The token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped"
    );
    supported("Helm of the Host");
    let mut t = TestGame::new(2);
    let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    attach_new(&mut t, P0, "Helm of the Host", kiki);
    attach_new(&mut t, P0, "Holy Strength", kiki);
    dress_up(&mut t, kiki);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert!(fresh(&t, tok));
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::single(Color::Red));
    // The exception: the token isn't legendary (so both stay).
    assert!(!t.obj_now(tok).chars.supertypes.contains(Supertype::Legendary));
    assert!(t.on_battlefield(kiki));
}

#[test]
fn nightmare_shepherd_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Nightmare Shepherd",
        "The token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else; see below). It doesn't copy whether that creature was tapped or untapped"
    );
    supported("Nightmare Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightmare Shepherd");
    let angel = t.battlefield(P0, "Serra Angel");
    attach_new(&mut t, P0, "Holy Strength", angel);
    dress_up(&mut t, angel);
    let before = t.g.battlefield.clone();
    t.answer_yes(P0, true);
    destroy(&mut t, angel);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Serra Angel");
    assert!(fresh(&t, toks[0]));
    // The exceptions: 1/1 and a Nightmare; the printed color, not the changed one.
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(tok.chars.subtypes.iter().any(|s| s == "Nightmare"));
    assert!(tok.chars.subtypes.iter().any(|s| s == "Angel"));
    assert_eq!(tok.chars.colors, ColorSet::single(Color::White));
    assert!(tok.has_keyword(KeywordKind::Flying));
}

#[test]
fn necroduality_token_copies_only_the_printed_zombie() {
    cr!("707.2", "707.2a");
    ruling!(
        "Necroduality",
        "The token copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else; see below)."
    );
    supported("Necroduality");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necroduality");
    let zombies = t.enter(P0, "Scathe Zombies");
    t.settle();
    dress_up(&mut t, zombies);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Scathe Zombies");
    assert!(fresh(&t, toks[0]));
    assert_eq!(t.pt(toks[0]), (2, 2));
    assert_eq!(t.obj_now(toks[0]).chars.colors, ColorSet::single(Color::Black));
}

#[test]
fn oltec_matterweaver_token_copies_the_original_token_characteristics() {
    cr!("707.2", "111.4");
    ruling!(
        "Oltec Matterweaver",
        "The new token copies the original characteristics of the target token as stated by the effect that created the target token and nothing else"
    );
    supported("Oltec Matterweaver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oltec Matterweaver");
    t.lands(P0, "Forest", 4);
    // First creature spell: a 1/1 Gnome artifact creature token.
    let before = t.g.battlefield.clone();
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.cast(P0, bears).go();
    t.resolve_all();
    let gnome = new_tokens(&t, P0, &before)[0];
    assert_eq!(t.pt(gnome), (1, 1));
    dress_up(&mut t, gnome);
    assert_eq!(t.pt(gnome), (6, 6));
    // Second: a copy of that Gnome, which copies only what the creating effect set.
    let before = t.g.battlefield.clone();
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(gnome)]);
    t.cast(P0, bears).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Gnome Token");
    assert!(fresh(&t, toks[0]));
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(tok.chars.colors.is_colorless());
    assert!(tok.is(CardType::Artifact) && tok.is(CardType::Creature));
}

#[test]
fn wire_surgeons_encore_tokens_copy_only_the_card() {
    cr!("707.2", "702.141a");
    ruling!(
        "Wire Surgeons",
        "The tokens copy only what’s on the original card. Effects that modified that creature when it was previously on the battlefield won’t be copied."
    );
    supported("Wire Surgeons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wire Surgeons");
    let thopter = t.battlefield(P0, "Ornithopter");
    attach_new(&mut t, P0, "Holy Strength", thopter);
    dress_up(&mut t, thopter);
    destroy(&mut t, thopter);
    let card = t.g.current(thopter);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    let before = t.g.battlefield.clone();
    activate_named(&mut t, P0, card, "Encore", 0).expect("encore");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Ornithopter");
    assert!(fresh(&t, toks[0]));
    assert_eq!(t.pt(toks[0]), (0, 2));
    assert!(t.obj_now(toks[0]).chars.colors.is_colorless());
}
