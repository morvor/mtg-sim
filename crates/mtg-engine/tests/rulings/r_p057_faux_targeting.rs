//! Rulings batch P057 — "faux targeting": a player chosen as a permanent enters (CR
//! 614.12, 607.2d) rather than targeted. Cursed Rack's chosen player gets a maximum hand
//! size of four (CR 402.2, 514.1); the choice stays with the permanent when it changes
//! controllers, and the permanent just does nothing once that player has left (CR 800.4a).
//! Black Vise and The Rack trigger at the beginning of the chosen player's upkeep.

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

/// Advances in turn order to `p`'s upkeep, resolves what triggered, and returns (`p`'s
/// hand size then, the life `p` lost since the previous call).
fn upkeep_of(t: &mut TestGame, p: PlayerId, life_before: i32) -> (usize, i32) {
    t.advance_to(p, Step::Upkeep);
    let hand = t.hand_size(p);
    t.resolve_all();
    (hand, life_before - t.life(p))
}

fn give_cards(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.hand(p, "Grizzly Bears");
    }
}

/// Black Vise ("At the beginning of the chosen player's upkeep, this artifact deals X
/// damage to that player, where X is the number of cards in their hand minus 4.") and The
/// Rack ("... where X is 3 minus the number of cards in their hand."): only the chosen
/// player, through a control change, and nobody once that player has left the game.
#[test]
fn black_vise_and_the_rack_keep_their_chosen_player() {
    cr!("607.2d", "800.4a", "107.1b");
    ruling!(
        "Black Vise",
        "You choose one opposing player as it enters and it only affects that one player. This choice is not changed even if Black Vise changes controllers. It becomes useless but stays on the battlefield if that player leaves the game."
    );
    ruling!(
        "The Rack",
        "You choose one opposing player as this card enters, and it only affects that one player. This choice is not changed even if this card changes controllers. It becomes useless but stays on the battlefield if the chosen player leaves the game."
    );
    supported("Black Vise");
    supported("The Rack");
    let vise = |hand: usize| (hand as i32 - 4).max(0);
    let rack = |hand: usize| (3 - hand as i32).max(0);
    for (name, cards, dmg) in [
        ("Black Vise", 7, &vise as &dyn Fn(usize) -> i32),
        ("The Rack", 0, &rack),
    ] {
        let mut t = TestGame::new(3);
        t.answer_choose(P0, &[Entity::Player(P1)]);
        let a = t.enter(P0, name);
        give_cards(&mut t, P1, cards);
        give_cards(&mut t, P2, cards);
        // The chosen player holds enough (or few enough) cards for it to deal damage;
        // so does the other opponent.
        let (hand, lost) = upkeep_of(&mut t, P1, 20);
        assert!(lost == dmg(hand) && lost > 0, "{name}: {hand} cards, lost {lost}");
        let (hand, lost) = upkeep_of(&mut t, P2, 20);
        assert!(dmg(hand) > 0 && lost == 0, "{name}: only the chosen player");
        assert_eq!(upkeep_of(&mut t, P0, 20).1, 0, "{name}");
        // P2 gains control of it: still P1 (not P2's opponents).
        give_control(&mut t, a, P2);
        let life = t.life(P1);
        let (hand, lost) = upkeep_of(&mut t, P1, life);
        assert!(lost == dmg(hand) && lost > 0, "{name}: {hand} cards, lost {lost}");
        assert_eq!(upkeep_of(&mut t, P2, 20).1, 0, "{name}");
        assert_eq!(upkeep_of(&mut t, P0, 20).1, 0, "{name}");
        // P1 leaves: it stays and does nothing.
        t.g.lose_game(P1);
        t.settle();
        assert!(t.on_battlefield(a), "{name}");
        let (hand, lost) = upkeep_of(&mut t, P2, 20);
        assert!(dmg(hand) > 0 && lost == 0, "{name}");
        assert_eq!(upkeep_of(&mut t, P0, 20).1, 0, "{name}");
        assert_eq!((t.life(P0), t.life(P2)), (20, 20), "{name}");
    }
}
