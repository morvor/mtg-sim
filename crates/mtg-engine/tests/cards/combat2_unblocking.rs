//! Removing a blocker from combat and making the creatures it was blocking unblocked,
//! and blocking by an effect (CR 506.4, 509.1h, 509.3): False Orders, Imprison, Balduvian
//! Warlord.

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn blocked(t: &TestGame, a: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(a))
}

fn blocking(t: &TestGame, b: ObjectId) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.blocking(b))
        .unwrap_or_default()
}

/// P0 attacks P1 with two Grizzly Bears; P1 blocks the first with a Wall of Wood.
/// Returns (bears a, bears b, wall).
fn bears_and_wall(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    declare_blocks(t, P1, &[(wall, a)]);
    (a, b, wall)
}

#[test]
fn false_orders_unblocks_and_reblocks_another_attacker() {
    cr!("506.4", "509.1h", "509.3a");
    supported("False Orders");
    let mut t = TestGame::new(2);
    let (a, b, wall) = bears_and_wall(&mut t);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "False Orders");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.cast(P0, spell).target(wall).go();
    t.resolve();
    assert!(!blocked(&t, a));
    assert!(blocked(&t, b));
    assert_eq!(blocking(&t, wall), vec![b]);
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.g.obj(wall).damage, 2);
}

#[test]
fn false_orders_without_the_reblock_leaves_the_creature_out_of_combat() {
    cr!("506.4", "509.1h");
    ruling!(
        "False Orders",
        "If a creature is removed from being a blocker of a given attacker, any triggered abilities that would have happened because it was declared as a blocker still happen."
    );
    let mut t = TestGame::new(2);
    let (a, b, wall) = bears_and_wall(&mut t);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "False Orders");
    t.answer_yes(P0, false);
    t.cast(P0, spell).target(wall).go();
    t.resolve();
    assert!(!blocked(&t, a) && !blocked(&t, b));
    assert!(!t.g.is_blocking(wall));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.g.obj(wall).damage, 0);
}

#[test]
fn false_orders_only_unblocks_creatures_that_creature_alone_blocked() {
    cr!("509.1h");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(w1, a), (w2, a)]);
    // The second Wall leaves the battlefield: it still blocked the Bears this combat.
    t.g.destroy(w2, None);
    t.settle();
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "False Orders");
    t.answer_yes(P0, false);
    t.cast(P0, spell).target(w1).go();
    t.resolve();
    assert!(blocked(&t, a));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn balduvian_warlord_moves_a_blocker_to_another_attacker() {
    cr!("506.4", "509.1h", "509.3a");
    ruling!(
        "Balduvian Warlord",
        "If an attacking creature becomes unblocked this way and remains unblocked, it will deal its combat damage to the defending player."
    );
    supported("Balduvian Warlord");
    let mut t = TestGame::new(2);
    let warlord = t.battlefield(P1, "Balduvian Warlord");
    let (a, b, wall) = bears_and_wall(&mut t);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.activate(P1, warlord, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve();
    assert!(!blocked(&t, a));
    assert!(blocked(&t, b));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn balduvian_warlord_can_reblock_the_same_attacker() {
    cr!("509.1h", "509.3a");
    ruling!(
        "Balduvian Warlord",
        "The targeted creature that’s removed from combat may then block the attacking creature that it was originally blocking. If so, that creature will no longer be unblocked."
    );
    let mut t = TestGame::new(2);
    let warlord = t.battlefield(P1, "Balduvian Warlord");
    let (a, b, wall) = bears_and_wall(&mut t);
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.activate(P1, warlord, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve();
    assert!(blocked(&t, a));
    assert!(!blocked(&t, b));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn balduvian_warlord_doesnt_unblock_a_creature_another_creature_blocked() {
    cr!("509.1h");
    ruling!(
        "Balduvian Warlord",
        "The attacking creature will become unblocked only if the targeted creature was the only one that blocked it this combat, even if all the other creatures that were blocking it are no longer on the battlefield or are no longer blocking it"
    );
    let mut t = TestGame::new(2);
    let warlord = t.battlefield(P1, "Balduvian Warlord");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    declare_blocks(&mut t, P1, &[(w1, a), (w2, a)]);
    t.g.destroy(w2, None);
    t.settle();
    // The first Wall moves to the other Bears; the first Bears stay blocked (by no
    // creature), so neither deals damage to P1.
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.activate(P1, warlord, 0, &[Entity::Object(w1)]).unwrap();
    t.resolve();
    assert!(blocked(&t, a));
    assert_eq!(blocking(&t, w1), vec![b]);
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn imprison_paid_removes_the_blocker_and_unblocks() {
    cr!("506.4", "509.1h");
    supported("Imprison");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let aura = t.battlefield(P0, "Imprison");
    assert!(t.g.attach(aura, Entity::Object(wall)));
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    declare_blocks(&mut t, P1, &[(wall, a)]);
    t.resolve_all();
    assert!(t.obj_now(wall).tapped);
    assert!(!t.g.is_blocking(wall));
    assert!(!blocked(&t, a));
    assert!(t.on_battlefield(aura));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn imprison_unpaid_is_destroyed() {
    cr!("118.12");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let aura = t.battlefield(P0, "Imprison");
    assert!(t.g.attach(aura, Entity::Object(wall)));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(aura));
    assert!(blocked(&t, a));
    assert!(t.g.is_blocking(wall));
}
