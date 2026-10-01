//! Rulings batch P057 — "faux targeting": a player chosen as a permanent enters (CR
//! 614.12, 607.2d) rather than targeted. Cursed Rack's chosen player gets a maximum hand
//! size of four (CR 402.2, 514.1); the choice stays with the permanent when it changes
//! controllers, and the permanent just does nothing once that player has left (CR 800.4a).

use crate::r_s01_common::supported;
use crate::r_s06_common::give_control;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn max_hand(t: &mut TestGame, p: PlayerId) -> Option<i32> {
    t.g.recompute();
    t.g.player(p).max_hand_size
}

#[test]
fn cursed_rack_makes_the_chosen_player_discard_down_to_four() {
    cr!("402.2", "514.1", "607.2d");
    ruling!(
        "Cursed Rack",
        "The effect of lowering the maximum hand size makes it so the player discards down to 4 cards (instead of the usual 7) during the cleanup step."
    );
    supported("Cursed Rack");
    // "As this artifact enters, choose an opponent. / The chosen player's maximum hand
    // size is four."
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Cursed Rack");
    assert_eq!(max_hand(&mut t, P1), Some(4));
    assert_eq!(max_hand(&mut t, P0), Some(7));
    for _ in 0..6 {
        t.hand(P1, "Grizzly Bears");
        t.hand(P0, "Grizzly Bears");
    }
    // P1's cleanup step: down to 4.
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to(P1, Step::End);
    let p1_before = t.hand_size(P1);
    assert_eq!(p1_before, 6);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 4);
    // P0 (not chosen) keeps 6 cards through their own cleanup step.
    t.set_step(P0, Step::PostcombatMain);
    let p0 = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), p0);
}

#[test]
fn cursed_rack_keeps_its_choice_under_a_new_controller_and_stays_when_the_player_leaves() {
    cr!("607.2d", "800.4a");
    ruling!(
        "Cursed Rack",
        "You choose one opposing player just as this card is entering and it only affects that one player. This choice is not changed even if this card changes controllers. It becomes useless but stays on the battlefield if the chosen player leaves the game."
    );
    supported("Cursed Rack");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let rack = t.enter(P0, "Cursed Rack");
    assert_eq!(max_hand(&mut t, P1), Some(4));
    assert_eq!(max_hand(&mut t, P2), Some(7), "only the chosen player");
    // P1 gains control of it: still P1's hand size, not P1's opponents'.
    give_control(&mut t, rack, P1);
    assert_eq!(t.obj_now(rack).controller, P1);
    assert_eq!(max_hand(&mut t, P1), Some(4));
    assert_eq!(max_hand(&mut t, P0), Some(7));
    assert_eq!(max_hand(&mut t, P2), Some(7));
    // Back to P0; then P1 leaves the game: the Rack stays and affects no one.
    give_control(&mut t, rack, P0);
    t.g.lose_game(P1);
    t.settle();
    assert!(t.on_battlefield(rack));
    assert_eq!(max_hand(&mut t, P0), Some(7));
    assert_eq!(max_hand(&mut t, P2), Some(7));
}
