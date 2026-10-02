//! Rulings batch P122 — in an instant or sorcery with a single target, "other creatures"
//! means creatures other than that target (the spell itself is never one of them):
//! Intimidation Bolt (tested in `r_p122_targets.rs`), Exhilarating Elocution, Phalanx
//! Tactics, Moment of Glory, Chandra's Ignition and Chain Assassination. Each test fails
//! if "other" is read as "other than the spell".

use crate::r_p122_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s28_common::cast_card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn exhilarating_elocution_other_creatures_exclude_the_target() {
    cr!("608.2c", "122.1a");
    supported("Exhilarating Elocution");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Exhilarating Elocution");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4), "two counters, not the +1/+1");
    assert_eq!(t.pt(lions), (3, 2));
    assert_eq!(t.pt(theirs), (3, 3), "only creatures you control");
}

#[test]
fn phalanx_tactics_each_other_creature_excludes_the_target() {
    cr!("608.2c", "611.2c");
    supported("Phalanx Tactics");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Phalanx Tactics");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 3));
    assert_eq!(t.pt(lions), (3, 2));
}

#[test]
fn moment_of_glory_from_a_graveyard() {
    cr!("608.2c", "702.34a");
    supported("Moment of Glory");
    // From the hand: only the target gets a counter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Moment of Glory");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(lions, "+1/+1"), 0);
    // With flashback: each other creature you control gets one too, but not the target
    // a second time.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let theirs = t.battlefield(P1, "Hill Giant");
    let card = t.graveyard(P0, "Moment of Glory");
    t.lands(P0, "Plains", 5);
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(obj(bears))
        .go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(lions, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn chandras_ignition_doesnt_damage_the_target() {
    cr!("608.2c", "120.3");
    supported("Chandra's Ignition");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(giant)]);
    cast_card(&mut t, P0, "Chandra's Ignition");
    t.resolve_all();
    assert!(t.on_battlefield(giant), "it doesn't deal damage to itself");
    assert_eq!(t.obj_now(giant).damage, 0);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn chain_assassination_another_creature_than_the_target() {
    cr!("608.2c", "700.4");
    supported("Chain Assassination");
    // Only the target died: no card.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Chain Assassination");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
    // Another creature died this turn: draw a card.
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    destroy(&mut t, lions);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Chain Assassination");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
