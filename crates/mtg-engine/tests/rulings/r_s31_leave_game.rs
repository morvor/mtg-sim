//! Rulings batch S31 — leaving a multiplayer game (CR 800.4a): the cards a player owns
//! leave with them, objects they control but don't own are exiled, and cards others own
//! that they exiled face down stay exiled, face down (CR 406.3).

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s04_common::add_mana;
use mtg_engine::decision::Action;
use mtg_engine::facedown;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
    t.settle();
}

/// P0 casts Reanimate ("Put target creature card from a graveyard onto the battlefield
/// under your control. You lose life equal to that card's mana value.") on P1's Hill
/// Giant in a three-player game. Returns the game and the Giant.
fn reanimated_giant() -> (TestGame, ObjectId) {
    supported("Reanimate");
    let mut t = TestGame::new(3);
    let giant = t.graveyard(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::B, 1);
    let spell = t.hand(P0, "Reanimate");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.life(P0), 16);
    (t, giant)
}

#[test]
fn a_reanimated_creature_is_exiled_when_its_controller_leaves_the_game() {
    cr!("800.4a");
    ruling!(
        "Reanimate",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the creature you control from Reanimate is exiled."
    );
    // P0 leaves: the Giant P1 owns is exiled.
    let (mut t, giant) = reanimated_giant();
    concede(&mut t, P0);
    assert!(!t.g.player(P0).in_game());
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(t.obj_now(giant).owner, P1);
    // P1 (its owner) leaves instead: the Giant leaves the game with them.
    let (mut t, giant) = reanimated_giant();
    concede(&mut t, P1);
    assert!(!t.on_battlefield(giant));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(!t.in_exile("Hill Giant"));
}

#[test]
fn edward_kenways_face_down_cards_stay_exiled_after_its_controller_leaves() {
    cr!("800.4a", "406.3");
    ruling!(
        "Edward Kenway",
        "If you leave the game, the cards remain exiled face down indefinitely. No player may look at them."
    );
    supported("Edward Kenway");
    supported("Smuggler's Copter");
    // Edward Kenway: "Whenever a Vehicle you control deals combat damage to a player, look
    // at the top card of that player's library, then exile it face down. You may play that
    // card for as long as it remains exiled." Smuggler's Copter (a 3/3 flying Vehicle,
    // crew 1) is crewed by Grizzly Bears and attacks P1.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Edward Kenway");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.library_top(P1, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, copter, 0, &[]).expect("crew 1");
    t.resolve_all();
    t.clear_answers();
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(copter, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(card), Zone::Exile);
    let exiled = t.g.current(card);
    assert!(t.obj_now(card).face_down);
    assert!(facedown::can_look_at(&t.g, P0, exiled));
    // P0 leaves the game: the card P1 owns stays exiled face down, and nobody may look at
    // it.
    concede(&mut t, P0);
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.g.current(card), exiled);
    assert!(t.obj_now(card).face_down);
    for p in [P1, P2] {
        assert!(!facedown::can_look_at(&t.g, p, exiled));
    }
}

#[test]
fn dream_thiefs_bandanas_face_down_cards_stay_exiled_after_its_controller_leaves() {
    cr!("800.4a", "406.3", "702.6a");
    ruling!(
        "Dream-Thief's Bandana",
        "If you leave the game, any remaining face-down exiled cards remain exiled face down indefinitely. No player may look at them."
    );
    supported("Dream-Thief's Bandana");
    // "Whenever equipped creature deals combat damage to a player, look at the top card of
    // their library, then exile it face down. For as long as it remains exiled, you may
    // play it, and mana of any type can be spent to cast that spell. Equip {1}"
    let mut t = TestGame::new(3);
    let bandana = t.battlefield(P0, "Dream-Thief's Bandana");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.library_top(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, bandana, 0, &[Entity::Object(bears)])
        .expect("equip");
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(card), Zone::Exile);
    let exiled = t.g.current(card);
    assert!(t.obj_now(card).face_down);
    assert!(facedown::can_look_at(&t.g, P0, exiled));
    concede(&mut t, P0);
    assert_eq!(t.g.current(card), exiled);
    assert!(t.obj_now(card).face_down);
    for p in [P1, P2] {
        assert!(!facedown::can_look_at(&t.g, p, exiled));
    }
}
