//! Choices made by a player other than the controller in older Oracle wordings
//! (cards4-paper-effects, family `chooser-phrases`): "any target of their choice", "a
//! land of an opponent's choice", "attach ~ to a land of their choice", "Target opponent
//! chooses one.", "a third of ... Round up each time."

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn pandemonium_the_entering_creatures_controller_chooses_the_target() {
    cr!("115.1", "601.2c", "603.3d");
    ruling!(
        "Pandemonium",
        "player A controls the triggered ability even though player B chooses the target and chooses whether to have the creature deal damage"
    );
    supported("Pandemonium");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Pandemonium");
    let mine = t.battlefield(P0, "Grizzly Bears");
    // P1's creature enters: P1 decides and targets P0's Bears.
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(mine)]);
    let giant = t.enter(P1, "Hill Giant");
    t.g.flush_events();
    t.settle();
    // P0 controls the ability.
    let top = *t.g.stack.last().expect("trigger on the stack");
    assert_eq!(t.g.obj(top).controller, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    // P0 wasn't asked for the target.
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
}

#[test]
fn pandemonium_the_creatures_controller_may_decline() {
    cr!("603.5");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Pandemonium");
    t.answer_yes(P1, false);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.enter(P1, "Hill Giant");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn demonic_hordes_an_opponent_chooses_the_land_you_sacrifice() {
    cr!("701.21a", "118.12");
    interp!("opponents-choice-untargeted");
    supported("Demonic Hordes");
    let mut t = TestGame::new(2);
    let hordes = t.battlefield(P0, "Demonic Hordes");
    let a = t.battlefield(P0, "Swamp");
    let b = t.battlefield(P0, "Forest");
    t.answer_yes(P0, false);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(hordes).tapped);
    assert!(t.on_battlefield(a));
    assert!(t.in_graveyard(P0, "Forest"));
    // Paying {B}{B}{B}: nothing happens.
    let mut t = TestGame::new(2);
    let hordes = t.battlefield(P0, "Demonic Hordes");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.obj_now(hordes).tapped);
    assert_eq!(t.g.player(P0).graveyard.len(), 0);
}

#[test]
fn kudzu_moves_to_a_land_the_destroyed_lands_controller_chooses() {
    cr!("701.3a", "701.3b");
    ruling!(
        "Kudzu",
        "You can move it to any other player's land whenever you get to move it."
    );
    supported("Kudzu");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let kudzu = t.battlefield(P0, "Kudzu");
    let forest = t.battlefield(P1, "Forest");
    let plains = t.battlefield(P0, "Plains");
    t.g.attach(kudzu, Entity::Object(forest));
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(plains)]);
    t.g.tap(forest);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"));
    let k = t.obj_now(kudzu);
    assert_eq!(k.attached_to, Some(Entity::Object(plains)));
    assert_eq!(k.controller, P0);
}

#[test]
fn intuition_the_opponent_chooses_the_card_you_get() {
    cr!("701.23a", "608.2d");
    ruling!(
        "Intuition",
        "Targets an opponent. You choose an opponent during announcement."
    );
    supported("Intuition");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Intuition");
    t.lands(P0, "Island", 3);
    let a = t.library_top(P0, "Lightning Bolt");
    let b = t.library_top(P0, "Counterspell");
    let c = t.library_top(P0, "Dark Ritual");
    t.answer_choose(
        P0,
        &[Entity::Object(a), Entity::Object(b), Entity::Object(c)],
    );
    t.answer_choose_moved(P1, &[Entity::Object(c)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert!(t.in_hand(P0, "Dark Ritual"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Counterspell"));
}

#[test]
fn pox_takes_a_third_rounded_up_of_everything() {
    cr!("107.1a");
    ruling!(
        "Pox",
        "The losses, discards, and sacrifices are rounded up. For example, if you have 5 life, you'll lose 2 life"
    );
    supported("Pox");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let pox = t.hand(P0, "Pox");
    t.lands(P0, "Swamp", 4);
    t.g.players[1].life = 5;
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    for _ in 0..2 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.lands(P1, "Forest", 7);
    t.cast(P0, pox).go();
    t.resolve();
    // 5 life: lose 2. 4 cards: discard 2. 2 creatures: sacrifice 1. 7 lands: sacrifice 3.
    assert_eq!(t.life(P1), 3);
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Forest").len(), 4);
    // P0: 20 life → lose 7; 4 Swamps → sacrifice 2 (Pox's own hand is empty).
    assert_eq!(t.life(P0), 13);
    assert_eq!(t.named_on_battlefield("Swamp").len(), 2);
}
