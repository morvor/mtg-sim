//! Rulings batch P109 — Torment of Hailfire: "Repeat the following process X times. Each
//! opponent loses 3 life unless that player sacrifices a nonland permanent of their choice
//! or discards a card." Each repetition is a separate punisher choice (CR 118.12, 608.2c);
//! no state-based actions in between (CR 704.3); triggers look at each event as it happened
//! (CR 603.10a).

use crate::r_p009_common::obj;
use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Torment of Hailfire with X = `x` (answers queued beforehand).
fn hailfire(t: &mut TestGame, x: i64) {
    supported("Torment of Hailfire");
    lands_for_cost(t, P0, "Torment of Hailfire");
    t.lands(P0, "Swamp", x as usize);
    let c = t.hand(P0, "Torment of Hailfire");
    t.cast(P0, c).x(x).go();
    t.resolve();
}

/// `p` sacrifices `id` this repetition.
fn sacrifice(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.answer_yes(p, true);
    t.answer_choose(p, &[obj(id)]);
}

/// `p` neither sacrifices nor discards this repetition (loses 3 life).
fn take_three(t: &mut TestGame, p: PlayerId) {
    t.answer_yes(p, false);
    t.answer_yes(p, false);
}

#[test]
fn x_zero_does_nothing_and_life_is_always_an_option() {
    cr!("107.3a", "118.12");
    ruling!(
        "Torment of Hailfire",
        "If X is 0, Torment of Hailfire resolves with no effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let from = t.asked().len();
    hailfire(&mut t, 0);
    assert_eq!(t.life(P1), 20);
    assert!(t.asked()[from..].iter().all(|(p, _)| *p != P1));
    assert!(t.in_graveyard(P0, "Torment of Hailfire"));

    ruling!(
        "Torment of Hailfire",
        "Your opponent can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    ruling!(
        "Torment of Hailfire",
        "Then that player discards that card, sacrifices that permanent, or loses 3 life, then repeats this process if it hasn’t been done X times yet."
    );
    // X = 3: sacrifice, discard, life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P1, "Hill Giant");
    sacrifice(&mut t, P1, bears);
    t.answer_yes(P1, false);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(card)]);
    take_three(&mut t, P1);
    hailfire(&mut t, 3);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn no_state_based_actions_between_repetitions() {
    cr!("704.3", "608.2c");
    ruling!(
        "Torment of Hailfire",
        "State-based actions aren’t checked in between repetitions of this process, so the game state may be a little unusual while making the choice. For example, a player may sacrifice a creature and then later sacrifice an Aura that was attached to that creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Holy Strength");
    t.g.attach(aura, obj(bears));
    sacrifice(&mut t, P1, bears);
    sacrifice(&mut t, P1, aura);
    hailfire(&mut t, 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert_eq!(t.life(P1), 20, "the Aura was still there to be sacrificed");
}

#[test]
fn each_repetition_is_done_separately() {
    cr!("603.10a", "608.2c");
    ruling!(
        "Torment of Hailfire",
        "Each repetition through this process is done separately. If an opponent sacrifices a creature with an ability that triggers when another creature dies, for example, it will see creatures that are sacrificed before it, but not those are sacrificed after it."
    );
    supported("Falkenrath Noble");
    // P1 sacrifices a Bears, then Falkenrath Noble ("Whenever this creature or another
    // creature dies, ..."), then another Bears: the Noble triggers twice, not three times.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let noble = t.battlefield(P1, "Falkenrath Noble");
    let b = t.battlefield(P1, "Grizzly Bears");
    sacrifice(&mut t, P1, a);
    sacrifice(&mut t, P1, noble);
    sacrifice(&mut t, P1, b);
    hailfire(&mut t, 3);
    t.settle();
    let noble_triggers = crate::r_s25_common::abilities_from(&t, noble).len();
    assert_eq!(noble_triggers, 2);
}

#[test]
fn each_opponent_chooses_in_turn_order_each_repetition() {
    cr!("101.4", "608.2c");
    ruling!(
        "Torment of Hailfire",
        "In a multiplayer game, each opponent in turn order makes their choice once, then all of the actions occur simultaneously, then they repeat this process if it hasn’t been done X times yet."
    );
    let mut t = TestGame::new(3);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    sacrifice(&mut t, P1, b1);
    take_three(&mut t, P2);
    take_three(&mut t, P1);
    take_three(&mut t, P2);
    let from = t.asked().len();
    hailfire(&mut t, 2);
    let mut order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p != P0 && matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    order.dedup();
    assert_eq!(order, vec![P1, P2, P1, P2]);
    assert!(!t.on_battlefield(b1));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 14);
    assert!(t.on_battlefield(b2));

    // The actions of one repetition happen together: P1's Falkenrath Noble ("Whenever
    // this creature or another creature dies") sacrificed by P1 sees the Grizzly Bears P2
    // sacrifices in the same repetition.
    supported("Falkenrath Noble");
    let mut t = TestGame::new(3);
    let noble = t.battlefield(P1, "Falkenrath Noble");
    let bears = t.battlefield(P2, "Grizzly Bears");
    sacrifice(&mut t, P1, noble);
    sacrifice(&mut t, P2, bears);
    hailfire(&mut t, 1);
    t.settle();
    assert!(!t.on_battlefield(noble) && !t.on_battlefield(bears));
    assert_eq!(crate::r_s25_common::abilities_from(&t, noble).len(), 2);
}

#[test]
fn two_headed_giant_both_opponents_choose_each_time() {
    cr!("810.9", "101.4");
    ruling!(
        "Torment of Hailfire",
        "In a Two-Headed Giant game, Torment of Hailfire causes both opponents to sacrifice a nonland permanent, discard a card, or lose 3 life X times. Your opponents can choose the same torment or different ones each time."
    );
    let mut t = crate::r_s33_common::two_headed_giant();
    let life = t.life(P2);
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P3, "Grizzly Bears");
    // Repetition 1: P2 sacrifices, P3 loses 3. Repetition 2: both lose 3.
    sacrifice(&mut t, P2, bears);
    take_three(&mut t, P3);
    take_three(&mut t, P3);
    hailfire(&mut t, 2);
    assert!(!t.on_battlefield(bears));
    // 3 (P3) + 3 (P2, with nothing left to sacrifice or discard) + 3 (P3) from the
    // team's shared life total.
    assert_eq!(t.life(P2), life - 9);
    assert_eq!(t.life(P3), life - 9);
}
