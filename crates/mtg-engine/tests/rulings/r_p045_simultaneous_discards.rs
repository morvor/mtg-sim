//! Rulings batch P045 — "each opponent discards a card" / "each player discards a card":
//! each player chooses in turn order starting with the active player (CR 101.4) without
//! revealing the choice (CR 101.4a), then all the chosen cards are discarded at the same
//! time (CR 701.9a).

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// In a four-player game during P2's turn, `name` (an "each opponent discards a card"
/// creature) enters under P0's control and its trigger resolves. Each other player has
/// two cards in hand. The opponents choose in the order P2 (the active player), P3, P1,
/// none of them sees a card leave a hand before everyone has chosen, then each has
/// discarded one card.
fn each_opponent_discards_in_turn_order(name: &str) {
    supported(name);
    let mut t = TestGame::new(4);
    t.set_step(P2, Step::PrecombatMain);
    for p in [P1, P2, P3] {
        bears_in_hand(&mut t, p, 2);
    }
    let seen: Vec<_> = [P1, P2, P3]
        .into_iter()
        .map(|p| watch_discards(&mut t, p))
        .collect();
    let from = t.asked().len();
    t.enter(P0, name);
    t.resolve_all();
    assert_eq!(discard_choosers(&t, from), vec![P2, P3, P1], "{name}");
    for s in &seen {
        let s = s.lock().unwrap();
        assert_eq!(s.len(), 1, "{name}");
        assert_eq!(s[0], vec![0, 2, 2, 2], "{name}: a card left a hand early");
    }
    assert_eq!(hand_sizes(&t.g), vec![0, 1, 1, 1], "{name}");
    for p in [P1, P2, P3] {
        assert!(t.in_graveyard(p, "Grizzly Bears"), "{name}");
    }
}

#[test]
fn each_opponent_chooses_in_turn_order_then_all_discard_at_once() {
    cr!("101.4", "101.4a", "701.9a", "603.3");
    ruling!(
        "Hecteyes",
        "As Hecteyes's ability resolves, the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a card in hand without revealing it"
    );
    each_opponent_discards_in_turn_order("Hecteyes");
    ruling!(
        "Elderfang Disciple",
        "first the next opponent in turn order (or, if it's an opponent's turn, the opponent whose turn it is) chooses a card in hand and sets it aside without revealing it"
    );
    each_opponent_discards_in_turn_order("Elderfang Disciple");
    ruling!(
        "Nezumi Informant",
        "To resolve Nezumi Informant’s ability in a multiplayer game, the next opponent in turn order"
    );
    each_opponent_discards_in_turn_order("Nezumi Informant");
}

#[test]
fn on_your_turn_the_next_opponent_in_turn_order_chooses_first() {
    cr!("101.4", "701.9a");
    ruling!(
        "Cavern Whisperer",
        "first the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses a card in hand without revealing it"
    );
    supported("Cavern Whisperer");
    // Cavern Whisperer: "Whenever this creature mutates, each opponent discards a card."
    // P0 casts it for its mutate cost onto Grizzly Bears during P0's own turn: the order
    // is P1, P2, P3.
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for p in [P1, P2, P3] {
        bears_in_hand(&mut t, p, 2);
    }
    let seen: Vec<_> = [P1, P2, P3]
        .into_iter()
        .map(|p| watch_discards(&mut t, p))
        .collect();
    let whisperer = t.hand(P0, "Cavern Whisperer");
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    t.g.players[0].mana_pool.add_type(ManaType::C, 3);
    t.cast(P0, whisperer)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(discard_choosers(&t, from), vec![P1, P2, P3]);
    for s in &seen {
        assert_eq!(s.lock().unwrap()[0], vec![0, 2, 2, 2]);
    }
    assert_eq!(hand_sizes(&t.g), vec![0, 1, 1, 1]);
}

/// P0 controls `name` ("each player discards ..."), whose ability resolves during P0's
/// turn in a three-player game where everyone has `hand` cards: P0 chooses first, then P1,
/// then P2, and every card is discarded only after the last choice.
fn each_player_discards_simultaneously(t: &mut TestGame, n: usize, hand: usize, what: &str) {
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| watch_discards(t, p))
        .collect();
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(discard_choosers(t, from), vec![P0, P1, P2], "{what}");
    for s in &seen {
        assert_eq!(s.lock().unwrap()[0], vec![hand; 3], "{what}: early discard");
    }
    assert_eq!(hand_sizes(&t.g), vec![hand - n; 3], "{what}");
}

#[test]
fn each_player_chooses_starting_with_the_active_player_then_all_discard() {
    cr!("101.4", "101.4a", "701.9a");
    ruling!(
        "Miasmic Mummy",
        "first the player whose turn it is chooses a card to discard, then each other player in turn order chooses a card to discard, then all of those cards are discarded simultaneously"
    );
    supported("Miasmic Mummy");
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        bears_in_hand(&mut t, p, 2);
    }
    t.enter(P0, "Miasmic Mummy");
    each_player_discards_simultaneously(&mut t, 1, 2, "Miasmic Mummy");

    ruling!(
        "Delirium Skeins",
        "first the player whose turn it is chooses three cards to discard, then each other player in turn order chooses three cards to discard"
    );
    supported("Delirium Skeins");
    let mut t = TestGame::new(3);
    let skeins = t.hand(P0, "Delirium Skeins");
    for p in [P0, P1, P2] {
        bears_in_hand(&mut t, p, 4);
    }
    give_mana_for(&mut t, P0, "Delirium Skeins");
    t.cast(P0, skeins).go();
    each_player_discards_simultaneously(&mut t, 3, 4, "Delirium Skeins");
}

#[test]
fn stronghold_rats_each_player_chooses_secretly_then_all_discard() {
    cr!("101.4", "701.9a", "510.3a");
    ruling!(
        "Stronghold Rats",
        "each player chooses a card in hand without revealing it, then all of the cards are discarded at the same time"
    );
    supported("Stronghold Rats");
    let mut t = TestGame::new(3);
    let rats = t.battlefield(P0, "Stronghold Rats");
    for p in [P0, P1, P2] {
        bears_in_hand(&mut t, p, 2);
    }
    attack_with(&mut t, &[(rats, Entity::Player(P1))]);
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| watch_discards(&mut t, p))
        .collect();
    let from = t.asked().len();
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(discard_choosers(&t, from), vec![P0, P1, P2]);
    for s in &seen {
        assert_eq!(s.lock().unwrap()[0], vec![2, 2, 2]);
    }
    assert_eq!(hand_sizes(&t.g), vec![1, 1, 1]);
}

#[test]
fn enemy_of_enlightenment_upkeep_discard_order_and_empty_hands() {
    cr!("101.4", "701.9a", "503.1a");
    ruling!(
        "Enemy of Enlightenment",
        "As the last ability resolves, first you choose a card in hand without revealing it, then each other player in turn order does the same"
    );
    ruling!(
        "Enemy of Enlightenment",
        "If any player can't discard a card, skip that player and each other player discards a card."
    );
    supported("Enemy of Enlightenment");
    // P1 controls Enemy of Enlightenment; its trigger resolves in P1's upkeep. P2 has no
    // cards in hand: P2 is skipped, P1 then P0 choose and discard.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Enemy of Enlightenment");
    bears_in_hand(&mut t, P0, 2);
    bears_in_hand(&mut t, P1, 2);
    t.advance_to(P1, Step::Upkeep);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(discard_choosers(&t, from), vec![P1, P0]);
    assert_eq!(hand_sizes(&t.g), vec![1, 1, 0]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
