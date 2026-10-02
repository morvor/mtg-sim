//! Rulings batch P062 — "sacrifice that permanent" / "sacrifice it": a player can sacrifice
//! only a permanent they control (CR 701.21a). The ability's controller (or the player an
//! instruction names: "that player sacrifices that creature") sacrifices it only if they
//! control it; "If you do" then sees that it wasn't sacrificed. Regression tests for the
//! cards whose "sacrifice that permanent" is compiled with that check.

use crate::r_p062_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Equipment `name` is on P1's Bears; P0 moves it to P0's Hill Giant: P1's Bears
/// aren't sacrificed. Moved on to P0's own Bears, the Giant is sacrificed.
fn unattached_from_opponents_creature(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let eq = attach_new(&mut t, P0, name, theirs);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, eq, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, eq), Some(obj(giant)));
    assert!(t.on_battlefield(theirs), "P0 sacrificed P1's creature");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    activate_containing(&mut t, P0, eq, "Equip").expect("equip");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn grafted_exoskeleton_cant_sacrifice_an_opponents_creature() {
    cr!("701.21a", "603.10c");
    unattached_from_opponents_creature("Grafted Exoskeleton");
}

#[test]
fn stitchers_graft_cant_sacrifice_an_opponents_creature() {
    cr!("701.21a", "603.10c");
    unattached_from_opponents_creature("Stitcher's Graft");
}

/// `attacker` (controlled by `ap`) attacks the other player unblocked; combat ends.
fn unblocked(t: &mut TestGame, ap: PlayerId, attacker: ObjectId) {
    let dp = if ap == P0 { P1 } else { P0 };
    t.g.combat = None;
    t.set_step(ap, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, Entity::Player(dp))]);
    block_and_finish(t, dp, &[]);
    t.resolve_all();
}

#[test]
fn foot_chopper_sacrifices_only_a_creature_you_control() {
    cr!("701.21a", "608.2c");
    supported("Foot Chopper");
    // P0's Foot Chopper on P0's Bears: P0 sacrifices them and draws two.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Foot Chopper", bears);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    unblocked(&mut t, P0, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // P0's Foot Chopper on P1's Bears, which deal combat damage to P0: P0 can't sacrifice
    // them, so P0 doesn't draw.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Foot Chopper", bears);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    unblocked(&mut t, P1, bears);
    assert_eq!(t.life(P0), 18);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn ogre_head_helm_sacrifices_only_a_creature_you_control() {
    cr!("701.21a", "608.2c");
    supported("Ogre-Head Helm");
    // P0's Helm on P0's Bears (4/4): P0 sacrifices them, discards, and draws three.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Ogre-Head Helm", bears);
    t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    unblocked(&mut t, P0, bears);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 3);
    // On P1's Bears: P0 can't sacrifice them; no discarding or drawing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Ogre-Head Helm", bears);
    t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    unblocked(&mut t, P1, bears);
    assert_eq!(t.life(P0), 16);
    assert!(t.on_battlefield(bears));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn ashlings_player_sacrifices_the_target_creature() {
    cr!("701.21a", "510.3a");
    supported("Ashling, the Extinguisher");
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling, the Extinguisher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    unblocked(&mut t, P0, ashling);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn star_athletes_target_is_sacrificed_by_its_controller_or_they_take_damage() {
    cr!("701.21a", "608.2c");
    supported("Star Athlete");
    // P1 sacrifices the target: no damage.
    let mut t = TestGame::new(2);
    let athlete = t.battlefield(P0, "Star Athlete");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P1, true);
    unblocked(&mut t, P0, athlete);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    // P1 doesn't: 5 damage to P1.
    let mut t = TestGame::new(2);
    let athlete = t.battlefield(P0, "Star Athlete");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P1, false);
    unblocked(&mut t, P0, athlete);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn slow_motions_player_sacrifices_the_creature_unless_they_pay() {
    cr!("701.21a", "118.12");
    supported("Slow Motion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Slow Motion", bears);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Slow Motion"));
}

#[test]
fn soul_tithes_player_sacrifices_the_permanent_unless_they_pay() {
    cr!("701.21a", "118.12");
    supported("Soul Tithe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Soul Tithe", bears);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn lingering_deaths_player_sacrifices_the_creature() {
    cr!("701.21a", "513.1");
    supported("Lingering Death");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Lingering Death", bears);
    // Not at P0's end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn fatal_grudge_opponents_sacrifice_a_permanent_sharing_a_type() {
    cr!("701.21a", "601.2h");
    supported("Fatal Grudge");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let grudge = t.hand(P0, "Fatal Grudge");
    t.answer_choose(P0, &[obj(mine)]);
    t.answer_choose(P1, &[obj(giant)]);
    let hand = t.hand_size(P0);
    t.cast_with(P0, grudge, &[]).expect("cast");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn burn_together_sacrifices_the_creature_you_control() {
    cr!("701.21a", "715.3d");
    supported("Callous Sell-Sword // Burn Together");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let card = t.hand(P0, "Callous Sell-Sword // Burn Together");
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .targets(&[obj(giant), obj(bears)])
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn special_moves_foot_toss_sacrifices_the_creature_you_control() {
    cr!("701.21a", "700.2");
    supported("Special Move");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Special Move");
    t.answer_targets(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, card).modes(&[0, 2]).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}
