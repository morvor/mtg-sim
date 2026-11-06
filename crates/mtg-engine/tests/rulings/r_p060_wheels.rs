//! Rulings batch P060 — "each player draws" effects and wheels: a player who draws more
//! cards than their library holds loses the next time state-based actions are checked
//! (CR 704.5b), and if all players lose at once the game is a draw (CR 104.4a); players
//! act in APNAP order and discards happen together (CR 101.4); shuffling an empty hand
//! still shuffles the library (CR 701.24).

use crate::r_p055_common::{all_pass, run_to_turn_of};
use crate::r_p060_common::*;
use crate::r_s02_common::create_token;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// A way to make every player draw: `run` sets the effect up and puts it on the stack (or
/// resolves it) in a game whose libraries hold `lib` cards each; the players then draw
/// more cards than that.
struct Wheel {
    card: &'static str,
    run: fn(&mut TestGame),
}

fn whispering_madness(t: &mut TestGame) {
    // P1 discards three: each player draws three.
    bears_in_hand(t, P1, 3);
    give_mana_for(t, P0, "Whispering Madness");
    let s = t.hand(P0, "Whispering Madness");
    t.cast(P0, s).go();
}

fn jace_ultimate(t: &mut TestGame) {
    let jace = pw(t, P0, "Jace, Memory Adept", 7);
    t.answer_targets(P0, &[pl(P0), pl(P1)]);
    t.activate(P0, jace, 2, &[]).unwrap();
}

fn folio(t: &mut TestGame) {
    let folio = t.battlefield(P0, "Folio of Fancies");
    t.lands(P0, "Wastes", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, folio, 0, &[]).unwrap();
}

fn prosperity(t: &mut TestGame) {
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let s = t.hand(P0, "Prosperity");
    t.cast(P0, s).x(3).go();
}

fn magus(t: &mut TestGame) {
    let magus = t.battlefield(P0, "Magus of the Wheel");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, magus, 0, &[]).unwrap();
}

fn great_aurora(t: &mut TestGame) {
    // Tokens are shuffled in and cease to exist, but count toward the cards drawn.
    for p in [P0, P1] {
        for _ in 0..3 {
            create_token(t, p, "Soldier");
        }
    }
    give_mana_for(t, P0, "The Great Aurora");
    let s = t.hand(P0, "The Great Aurora");
    t.cast(P0, s).go();
}

fn dragon_mage(t: &mut TestGame) {
    let mage = t.battlefield(P0, "Dragon Mage");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(mage, pl(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

const WHEELS: &[Wheel] = &[
    Wheel {
        card: "Whispering Madness",
        run: whispering_madness,
    },
    Wheel {
        card: "Jace, Memory Adept",
        run: jace_ultimate,
    },
    Wheel {
        card: "Folio of Fancies",
        run: folio,
    },
    Wheel {
        card: "Prosperity",
        run: prosperity,
    },
    Wheel {
        card: "Magus of the Wheel",
        run: magus,
    },
    Wheel {
        card: "The Great Aurora",
        run: great_aurora,
    },
    Wheel {
        card: "Dragon Mage",
        run: dragon_mage,
    },
];

/// Runs the wheel with P0's library holding `p0_lib` cards and P1's `p1_lib`. Returns the
/// game after the effect resolved and state-based actions were checked.
fn run_wheel(w: &Wheel, p0_lib: usize, p1_lib: usize) -> TestGame {
    supported(w.card);
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, p0_lib);
    library_of(&mut t, P1, p1_lib);
    (w.run)(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", w.card);
    t.g.resolve_top();
    // The players lose only when state-based actions are checked.
    assert!(!t.has_lost(P0) && !t.has_lost(P1), "{}", w.card);
    t.settle();
    t
}

#[test]
fn drawing_from_too_small_a_library_loses_and_all_losing_is_a_draw() {
    cr!("704.5b", "104.4a", "104.3c", "121.4");
    ruling!(
        "Whispering Madness",
        "Each player draws cards simultaneously. If this causes a player to draw more cards than are left in their library, that player will lose the game. If this causes all players to do so, the game is a draw."
    );
    ruling!(
        "Jace, Memory Adept",
        "If Jace’s third ability causes a player to draw more cards than are left in their library, that player loses the game as a state-based action. If this ability causes all players to do this, the game is a draw."
    );
    ruling!(
        "Folio of Fancies",
        "If a player has fewer than X cards in their library, that player loses the game after the first ability of Folio of Fancies resolves. If each player loses the game this way, the game is a draw."
    );
    ruling!(
        "Prosperity",
        "If both players run out of cards during this effect, the game is a draw."
    );
    ruling!(
        "Magus of the Wheel",
        "If each player has fewer than seven cards in their library as Magus of the Wheel's ability resolves, the game ends in a draw."
    );
    ruling!(
        "The Great Aurora",
        "If a player is required to draw more cards than their library contains, that player loses the game. If all players lose the game this way, the game is a draw."
    );
    ruling!(
        "Dragon Mage",
        "If a player has fewer than seven cards in their library, that player loses the game after Dragon Mage's triggered ability resolves. If each player loses the game this way, the game's a draw."
    );
    for w in WHEELS {
        // Each library is too small (Jace: 20 cards; the others draw three or seven).
        let small = if w.card == "Jace, Memory Adept" { 19 } else { 2 };
        let t = run_wheel(w, small, small);
        assert!(t.has_lost(P0) && t.has_lost(P1), "{}", w.card);
        assert!(is_draw(&t), "{}: a draw", w.card);
        // Only P1's library is too small: P1 loses, P0 wins.
        let t = run_wheel(w, 25, small);
        assert!(!t.has_lost(P0) && t.has_lost(P1), "{}", w.card);
        assert!(t.g.player(P0).has_won, "{}", w.card);
    }
}

#[test]
fn whispering_madness_multiplayer_losers_leave_and_the_game_goes_on() {
    cr!("704.5b", "800.4");
    ruling!(
        "Whispering Madness",
        "(In multiplayer games, multiple players drawing from an empty library will cause those players to lose although the game may continue.)"
    );
    let mut t = TestGame::new(4);
    library_of(&mut t, P1, 0);
    library_of(&mut t, P2, 0);
    whispering_madness(&mut t);
    t.resolve_all();
    assert!(t.has_lost(P1) && t.has_lost(P2));
    assert!(!t.has_lost(P0) && !t.has_lost(P3));
    assert!(!t.g.is_over(), "the game continues");
    // P0 and P3 drew three each.
    assert_eq!((t.hand_size(P0), t.hand_size(P3)), (3, 3));
}

/// The draw and discard events since event `from`: (player, is a draw).
fn draws_and_discards(t: &TestGame, from: usize) -> Vec<(PlayerId, bool)> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Drew { player, .. } => Some((*player, true)),
            Event::Discarded { player, .. } => Some((*player, false)),
            _ => None,
        })
        .collect()
}

#[test]
fn burning_inquiry_and_noggle_ransacker_draw_in_turn_order_then_discard_together() {
    cr!("101.4", "608.2e", "701.9b");
    ruling!(
        "Burning Inquiry",
        "First the player whose turn it is draws three cards, then each other player in turn order does so. Then the active player randomly chooses the three cards they will discard, then each other player in turn order does so. Then all the cards are discarded at the same time."
    );
    ruling!(
        "Noggle Ransacker",
        "First the player whose turn it is draws two cards, then each other player in turn order does so. Then the player whose turn it is randomly selects a card from their hand, then each other player in turn order does so. Finally, all cards selected this way are discarded at the same time."
    );
    // It's P1's turn (P1 is the active player); P0 casts or controls the card, so the
    // order starts with the active player, not with its controller.
    for (card, n, discard) in [("Burning Inquiry", 3, 3), ("Noggle Ransacker", 2, 1)] {
        supported(card);
        let mut t = TestGame::new(3);
        t.set_step(P1, Step::PrecombatMain);
        bears_in_hand(&mut t, P2, 1);
        let from = t.g.turn_events.len();
        let asked = t.asked().len();
        if card == "Burning Inquiry" {
            // Leyline of Anticipation lets P0 cast it during P1's turn.
            t.battlefield(P0, "Leyline of Anticipation");
            t.lands(P0, "Mountain", 1);
            let s = t.hand(P0, card);
            t.cast(P0, s).go();
            t.resolve_all();
        } else {
            t.enter(P0, card);
            t.resolve_all();
        }
        let ev = draws_and_discards(&t, from);
        let mut expect = Vec::new();
        for p in [P1, P2, P0] {
            expect.extend(std::iter::repeat_n((p, true), n));
        }
        assert_eq!(ev[..3 * n], expect[..], "{card}: draws in turn order from P1");
        // Then all discards, and nothing else in between.
        assert_eq!(ev.len(), 3 * n + 3 * discard, "{card}");
        assert!(ev[3 * n..].iter().all(|(_, d)| !d), "{card}");
        for p in [P0, P1, P2] {
            let k = ev[3 * n..].iter().filter(|(q, _)| *q == p).count();
            assert_eq!(k, discard, "{card}: {p} discards");
        }
        assert_eq!(t.hand_size(P2), 1 + n - discard, "{card}");
        // The cards are chosen at random: no player was asked to choose them.
        assert!(
            !t.asked()[asked..]
                .iter()
                .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })),
            "{card}: discarded at random"
        );
    }
}

#[test]
fn burning_inquiry_discards_three_even_after_drawing_fewer() {
    cr!("121.4", "104.3c", "701.9a");
    ruling!(
        "Burning Inquiry",
        "Each player must discard three cards, even a player that (for some reason) didn’t actually draw three cards."
    );
    // P1 has one card left in their library, Platinum Angel (they can't lose), and four
    // cards in hand: they draw one card and still discard three.
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, 1);
    t.battlefield(P1, "Platinum Angel");
    bears_in_hand(&mut t, P1, 4);
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Burning Inquiry");
    let before = libraries(&t);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![3, 1]);
    assert_eq!(t.hand_size(P1), 2, "4 + 1 drawn - 3 discarded");
    assert_eq!(t.graveyard_size(P1), 3);
    assert!(!t.has_lost(P1));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn echo_of_eons_isnt_shuffled_into_your_library_by_its_own_effect() {
    cr!("608.2n");
    ruling!(
        "Echo of Eons",
        "Echo of Eons won't be put into your graveyard until after it's finished resolving, which means it won't be shuffled into your library as part of its own effect."
    );
    supported("Echo of Eons");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 4);
    let echo = t.hand(P0, "Echo of Eons");
    t.cast(P0, echo).go();
    let lib = t.library_size(P0);
    t.resolve_all();
    // The Bears were shuffled in; Echo of Eons is in the graveyard (and can be flashed
    // back), not in the library.
    assert_eq!(t.library_size(P0), lib + 1 - 7);
    assert!(t.in_graveyard(P0, "Echo of Eons"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 7);
}

/// The players who shuffled their library since event `from`.
fn shufflers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Shuffled { player } => Some(*player),
            _ => None,
        })
        .collect()
}

#[test]
fn an_empty_hand_still_shuffles_the_library() {
    cr!("701.24a", "608.2e");
    ruling!(
        "Molten Psyche",
        "If a player doesn’t have any cards in their hand, the player will still shuffle their library."
    );
    ruling!(
        "Day's Undoing",
        "If both your hand and graveyard are empty as Day’s Undoing starts resolving, you’ll still shuffle your library before drawing seven cards."
    );
    // Molten Psyche: P0's hand is empty once it's cast; P1 holds two cards.
    supported("Molten Psyche");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 2);
    give_mana_for(&mut t, P0, "Molten Psyche");
    let s = t.hand(P0, "Molten Psyche");
    t.cast(P0, s).go();
    let from = t.g.turn_events.len();
    t.resolve_all();
    let mut sh = shufflers(&t, from);
    sh.sort();
    assert_eq!(sh, vec![P0, P1]);
    assert_eq!(hands(&t), vec![0, 2]);
    // Day's Undoing: P0's hand and graveyard are empty: P0 still shuffles, then draws.
    supported("Day's Undoing");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Day's Undoing");
    let s = t.hand(P0, "Day's Undoing");
    t.cast(P0, s).go();
    let from = t.g.turn_events.len();
    all_pass(&mut t, P0);
    let ev: Vec<(PlayerId, bool)> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Shuffled { player } if *player == P0 => Some((P0, false)),
            Event::Drew { player, .. } if *player == P0 => Some((P0, true)),
            _ => None,
        })
        .collect();
    assert_eq!(ev.len(), 8);
    assert_eq!(ev[0], (P0, false), "shuffled before drawing");
}

#[test]
fn days_undoing_triggers_while_ending_the_turn_go_on_the_stack_in_cleanup() {
    cr!("724.1b", "724.1f", "514.3a");
    ruling!(
        "Day's Undoing",
        "If any triggered abilities do trigger during the process of ending the turn, they’re put onto the stack during the cleanup step. If this happens, players will have a chance to cast spells and activate abilities, then there will be another cleanup step before the turn ends."
    );
    supported("Stonebinder's Familiar");
    let mut t = TestGame::new(2);
    // "Whenever one or more cards are put into exile during your turn, put a +1/+1 counter
    // on this creature." Day's Undoing is exiled as the turn ends (CR 724.1b).
    let familiar = t.battlefield(P0, "Stonebinder's Familiar");
    give_mana_for(&mut t, P0, "Day's Undoing");
    let s = t.hand(P0, "Day's Undoing");
    t.cast(P0, s).go();
    all_pass(&mut t, P0);
    assert!(t.in_exile("Day's Undoing"));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.counters(familiar, "+1/+1"), 0);
    let ok = t.g.run_until(50, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok, "no priority in the cleanup step");
    assert_eq!(t.g.turn.priority, Some(P0));
    let (steps, _) = run_to_turn_of(&mut t, P1);
    assert_eq!(t.counters(familiar, "+1/+1"), 1);
    assert_eq!(steps, vec![Step::Cleanup], "another cleanup step");
}

#[test]
fn players_with_one_or_no_cards_draw_none_with_dark_deal() {
    cr!("608.2c");
    ruling!(
        "Dark Deal",
        "If a player had one card or no cards in their hand, that player won't draw any cards."
    );
    supported("Dark Deal");
    // P0 has no other card, P1 one, P2 three.
    let mut t = TestGame::new(3);
    bears_in_hand(&mut t, P1, 1);
    bears_in_hand(&mut t, P2, 3);
    give_mana_for(&mut t, P0, "Dark Deal");
    let s = t.hand(P0, "Dark Deal");
    t.cast(P0, s).go();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 0, 2]);
    assert_eq!(hands(&t), vec![0, 0, 2]);
}

#[test]
fn runehorn_hellkite_is_exiled_as_a_cost_and_empty_hands_just_draw() {
    cr!("602.2b", "601.2h", "608.2e");
    ruling!(
        "Runehorn Hellkite",
        "Exiling Runehorn Hellkite from your graveyard is part of the cost of its activated ability. An opponent can't remove Runehorn Hellkite from your graveyard in response to you activating the ability."
    );
    ruling!(
        "Runehorn Hellkite",
        "If a player has no cards in hand, that player discards no cards, then draws seven cards."
    );
    supported("Runehorn Hellkite");
    let mut t = TestGame::new(2);
    let hk = t.graveyard(P0, "Runehorn Hellkite");
    t.lands(P0, "Mountain", 6);
    bears_in_hand(&mut t, P1, 2);
    crate::r_s06_common::activate_containing(&mut t, P0, hk, "Each player discards").unwrap();
    // Already exiled while the ability waits on the stack: nothing in the graveyard to
    // remove in response.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Runehorn Hellkite"));
    assert_eq!(t.graveyard_size(P0), 0);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![7, 7]);
    assert_eq!(hands(&t), vec![7, 7]);
    // P0 had no cards: discarded nothing. P1 discarded two.
    assert_eq!((t.graveyard_size(P0), t.graveyard_size(P1)), (0, 2));
}

#[test]
fn commit_puts_a_spell_or_permanent_into_its_owners_library() {
    cr!("608.2c", "401.7", "704.5d", "704.5e", "707.10", "111.7");
    ruling!(
        "Commit // Memory",
        "If the target's owner's library contains no cards, the target spell or permanent will be put into that player's library as the only card."
    );
    ruling!(
        "Commit // Memory",
        "If a spell is put into its owner's library, it's removed from the stack and thus will not resolve. The spell isn't countered; it just no longer exists. This works against a spell that can't be countered."
    );
    ruling!(
        "Commit // Memory",
        "If a copy of a spell or a token is put into its owner's library, it's moved there, then it will cease to exist as a state-based action."
    );
    supported("Commit // Memory");
    let commit = |t: &mut TestGame, target: Entity| {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 3);
        let c = t.hand(P0, "Commit // Memory");
        t.cast(P0, c)
            .method(CastMethod::Half(0))
            .target(target)
            .go();
    };
    // An empty library: the permanent becomes its only card.
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, 0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    commit(&mut t, obj(bears));
    t.resolve_all();
    assert_eq!(t.library_size(P1), 1);
    assert_eq!(t.zone(bears), Zone::Library(P1));
    // A spell that can't be countered (Abrupt Decay) is put into its owner's library: it
    // doesn't resolve, and isn't countered.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let decay = t.hand(P1, "Abrupt Decay");
    t.cast(P1, decay).target(ring).go();
    let lib = t.library_size(P1);
    commit(&mut t, obj(decay));
    let from = t.g.turn_events.len();
    t.resolve();
    assert_eq!(t.zone(decay), Zone::Library(P1));
    assert_eq!(t.library_size(P1), lib + 1);
    assert_eq!(t.stack_len(), 0, "removed from the stack");
    assert!(t.on_battlefield(ring), "Abrupt Decay didn't resolve");
    assert!(
        !t.g.turn_events[from..]
            .iter()
            .any(|e| matches!(e, Event::Countered { .. })),
        "not countered"
    );
    // A token goes into the library, then ceases to exist as a state-based action.
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P1, "Soldier");
    commit(&mut t, obj(token));
    t.settle();
    t.g.resolve_top();
    assert_eq!(t.zone(token), Zone::Library(P1), "moved there");
    t.settle();
    assert_eq!(t.zone(token), Zone::Nowhere, "then ceased to exist");
    // A copy of a spell: Expansion copies P1's Lightning Bolt; Commit puts the copy into
    // its owner's library, where it ceases to exist; the original still resolves.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(pl(P0)).go();
    t.lands(P0, "Island", 2);
    let exp = t.hand(P0, "Expansion // Explosion");
    t.cast(P0, exp)
        .method(CastMethod::Half(0))
        .target(bolt)
        .go();
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.stack_len(), 2, "the copy and the original");
    let copy = t.g.stack[1];
    assert_ne!(copy, bolt);
    commit(&mut t, obj(copy));
    t.settle();
    t.g.resolve_top();
    // The copy is owned by P0, who put it on the stack (CR 707.10).
    assert_eq!(t.zone(copy), Zone::Library(P0), "moved there");
    t.settle();
    assert_eq!(t.zone(copy), Zone::Nowhere, "then ceased to exist");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "only the original dealt damage");
}
