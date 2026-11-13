//! Rulings batch P006 — bottom-of-library manipulation: looking at more cards than the
//! library has (no draw, CR 121.4), drawing from a short library (CR 704.5b),
//! ordering cards put on the bottom (CR 401.4), what's revealed (CR 701.20), subtypes of
//! cards (CR 205.3), colorless lands (CR 105.2c, 202.2), casting from the graveyard with
//! a permission (CR 601.3, 601.2f, 202.3), priority after resolution (CR 117.3b), and
//! "draw, then put back" resolving at once (CR 608.2).

use crate::r_p006_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::can_cast;
use crate::r_s08_common::{legal_cast_methods, mana_value};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The cards revealed this turn (CR 701.20).
fn revealed(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == mtg_engine::reveal::REVEALED => *obj,
            _ => None,
        })
        .collect()
}

/// `p`'s library is exactly `top_first` (the first on top).
fn library_of(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    t.g.player_mut(p).library.clear();
    stack_library(t, p, top_first)
}

/// The bottom `n` cards of `p`'s library, the bottommost last.
fn bottom(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    let lib = &t.g.player(p).library;
    lib[..n].iter().rev().copied().collect()
}

#[test]
fn impulse_doesnt_shuffle() {
    cr!("401.4", "608.2c");
    ruling!(
        "Impulse",
        "Due to errata, you no longer shuffle your library."
    );
    let mut t = TestGame::new(2);
    let below = library_of(
        &mut t,
        P0,
        &["Forest", "Island", "Swamp", "Mountain", "Plains", "Hill Giant", "Craw Wurm"],
    );
    t.lands(P0, "Island", 2);
    let imp = t.hand(P0, "Impulse");
    t.cast(P0, imp).go();
    t.answer_choose(P0, &[obj(below[0])]);
    t.resolve_all();
    assert!(in_hand(&t, below[0]));
    // Hill Giant and Craw Wurm are now on top, untouched; the other three at the bottom.
    let lib = t.g.player(P0).library.clone();
    assert_eq!(lib.len(), 6);
    assert_eq!(lib[3..6].to_vec(), vec![below[6], below[5], below[4]]);
    let mut bottom3: Vec<_> = lib[..3].to_vec();
    bottom3.sort();
    let mut expected = below[1..4].to_vec();
    expected.sort();
    assert_eq!(bottom3, expected);
}

#[test]
fn worldly_counsel_you_choose_the_bottom_order() {
    cr!("401.4");
    ruling!(
        "Worldly Counsel",
        "You choose the order on the bottom of your library."
    );
    let mut t = TestGame::new(2);
    for l in ["Plains", "Island", "Swamp", "Mountain"] {
        t.battlefield(P0, l);
    }
    let cards = library_of(
        &mut t,
        P0,
        &["Hill Giant", "Grizzly Bears", "Craw Wurm", "Ornithopter", "Forest"],
    );
    let wc = t.hand(P0, "Worldly Counsel");
    t.cast(P0, wc).go();
    t.answer_choose(P0, &[obj(cards[0])]);
    // The three others listed top first: Ornithopter, Craw Wurm, Grizzly Bears.
    let from = mark(&t);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 1, 0]));
    t.resolve_all();
    let orders = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
        .count();
    assert_eq!(orders, 1, "P0 is asked for the order");
    // Answer [2, 1, 0] (top first): Ornithopter, Craw Wurm, then the Bears at the bottom.
    assert_eq!(bottom(&t, P0, 3), vec![cards[3], cards[2], cards[1]]);
}

#[test]
fn augur_of_bolas_with_a_short_library_and_what_it_reveals() {
    cr!("121.4", "701.20a", "704.5b");
    ruling!(
        "Augur of Bolas",
        "If there are fewer than three cards in your library when Augur of Bolas's ability resolves, you'll look at the remaining cards. This won't cause you to lose the game as you haven't drawn from an empty library."
    );
    ruling!(
        "Augur of Bolas",
        "You don't reveal the other two cards, nor do you reveal their order on the bottom of the library."
    );
    supported("Augur of Bolas");
    // Two cards: both looked at; P0 doesn't lose.
    let mut t = TestGame::new(2);
    let cards = library_of(&mut t, P0, &["Lightning Bolt", "Grizzly Bears"]);
    t.enter(P0, "Augur of Bolas");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cards[0])]);
    t.resolve_all();
    assert!(in_hand(&t, cards[0]));
    assert_eq!(t.library_size(P0), 1);
    assert!(!t.has_lost(P0));
    // Only the chosen instant is revealed.
    let mut t = TestGame::new(2);
    let cards = library_of(&mut t, P0, &["Lightning Bolt", "Grizzly Bears", "Hill Giant", "Forest"]);
    t.enter(P0, "Augur of Bolas");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cards[0])]);
    t.resolve_all();
    assert!(in_hand(&t, cards[0]));
    let shown = revealed(&t);
    assert!(shown.contains(&cards[0]), "the instant taken is revealed");
    for c in &cards[1..3] {
        assert!(!shown.contains(c) && !shown.contains(&t.g.current(*c)));
    }
}

#[test]
fn amass_the_components_with_a_short_library() {
    cr!("121.4", "704.5b");
    ruling!(
        "Amass the Components",
        "If you cast Amass the Components with fewer than three cards in your library, you’ll draw the remaining cards in your library, put a card from your hand on the bottom of your library, then lose the game for drawing a card from a library with no cards in it the next time state-based actions are performed."
    );
    let mut t = TestGame::new(2);
    let cards = library_of(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    t.lands(P0, "Island", 4);
    let amass = t.hand(P0, "Amass the Components");
    t.cast(P0, amass).go();
    t.answer_choose_moved(P0, &[obj(cards[1])]);
    t.g.resolve_top();
    // Before state-based actions: both drawn, one put back, P0 hasn't lost yet.
    assert!(!t.has_lost(P0));
    assert!(in_hand(&t, cards[0]));
    assert_eq!(t.g.player(P0).library, vec![t.g.current(cards[1])]);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn peer_through_depths_without_an_instant_puts_all_on_the_bottom() {
    cr!("401.4", "608.2c");
    ruling!(
        "Peer Through Depths",
        "If you don’t reveal an instant or sorcery card, put all the revealed cards on the bottom of your library in any order."
    );
    // There is one (Lightning Bolt), but P0 declines: all five go to the bottom.
    let mut t = TestGame::new(2);
    let cards = library_of(
        &mut t,
        P0,
        &["Lightning Bolt", "Grizzly Bears", "Hill Giant", "Forest", "Craw Wurm", "Island"],
    );
    t.lands(P0, "Island", 2);
    let peer = t.hand(P0, "Peer Through Depths");
    t.cast(P0, peer).go();
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.g.player(P0).library.last().copied(), Some(cards[5]), "Island on top");
    let mut b = bottom(&t, P0, 5);
    b.sort();
    let mut want = cards[..5].to_vec();
    want.sort();
    assert_eq!(b, want);
}

#[test]
fn fertile_thicket_may_leave_nothing_on_top() {
    cr!("401.4", "608.2d");
    ruling!(
        "Fertile Thicket",
        "You don’t have to reveal a basic land card from among the five cards you look at, even if one is there. If you don’t, you won’t put a card on top of your library."
    );
    supported("Fertile Thicket");
    let mut t = TestGame::new(2);
    let cards = library_of(
        &mut t,
        P0,
        &["Forest", "Grizzly Bears", "Hill Giant", "Lightning Bolt", "Craw Wurm", "Island"],
    );
    t.enter(P0, "Fertile Thicket");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.g.player(P0).library.last().copied(), Some(cards[5]), "Island on top");
    let mut b = bottom(&t, P0, 5);
    b.sort();
    let mut want = cards[..5].to_vec();
    want.sort();
    assert_eq!(b, want, "the Forest went to the bottom too");
    assert!(revealed(&t).is_empty());
}

#[test]
fn ancient_stirrings_lands_are_colorless() {
    cr!("105.2c", "202.2");
    ruling!(
        "Ancient Stirrings",
        "Objects with no mana cost, including lands that could produce colored mana, are colorless by default."
    );
    let mut t = TestGame::new(2);
    let cards = library_of(
        &mut t,
        P0,
        &["Grizzly Bears", "Forest", "Lightning Bolt", "Hill Giant", "Craw Wurm"],
    );
    t.lands(P0, "Forest", 1);
    let st = t.hand(P0, "Ancient Stirrings");
    t.cast(P0, st).go();
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cards[1])]);
    t.resolve_all();
    let c = choices_since(&t, P0, from);
    assert!(!c.is_empty());
    assert_eq!(c[0].0, vec![obj(cards[1])], "only the Forest is colorless");
    assert!(in_hand(&t, cards[1]));
}

#[test]
fn goblin_ringleader_takes_only_goblin_cards() {
    cr!("205.3", "608.2c");
    ruling!(
        "Goblin Ringleader",
        "If an effect refers to a “[subtype] spell” or “[subtype] card,” it refers only to a spell or card that has that subtype. For example, Goblin Gathering is a card that creates Goblins and features Goblins in its illustration, but it isn’t a Goblin card."
    );
    let mut t = TestGame::new(2);
    let cards = library_of(
        &mut t,
        P0,
        &["Goblin Gathering", "Mogg Fanatic", "Goblin Raider", "Hill Giant", "Forest"],
    );
    t.enter(P0, "Goblin Ringleader");
    t.resolve_all();
    assert!(in_hand(&t, cards[1]) && in_hand(&t, cards[2]));
    assert_eq!(t.zone(cards[0]), Zone::Library(P0), "Goblin Gathering isn't a Goblin");
    assert_eq!(t.zone(cards[3]), Zone::Library(P0));
    assert_eq!(t.g.player(P0).library.last().copied(), Some(t.g.current(cards[4])));
}

#[test]
fn stomping_slabs_needs_a_stomping_slabs() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Stomping Slabs",
        "If a card named Stomping Slabs wasn’t revealed this way, Stomping Slabs deals no damage to its target."
    );
    supported("Stomping Slabs");
    for (with, life) in [(false, 20), (true, 13)] {
        let mut t = TestGame::new(2);
        let mut names = vec!["Forest"; 8];
        if with {
            names[6] = "Stomping Slabs";
        }
        library_of(&mut t, P0, &names);
        t.lands(P0, "Mountain", 3);
        let slabs = t.hand(P0, "Stomping Slabs");
        t.cast(P0, slabs).target(pl(P1)).go();
        t.resolve_all();
        assert_eq!(t.life(P1), life, "with a Slabs among seven: {with}");
    }
}

#[test]
fn brainsurge_can_put_back_cards_from_the_old_hand() {
    cr!("608.2c", "401.4");
    ruling!(
        "Brainsurge",
        "The two cards you put on top of your library can be from the four you just drew or ones that were already in your hand."
    );
    let mut t = TestGame::new(2);
    let old1 = t.hand(P0, "Hill Giant");
    let old2 = t.hand(P0, "Craw Wurm");
    t.lands(P0, "Island", 3);
    let bs = t.hand(P0, "Brainsurge");
    t.cast(P0, bs).go();
    t.answer_choose(P0, &[obj(old1), obj(old2)]);
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    let top2 = [lib[lib.len() - 1], lib[lib.len() - 2]];
    assert!(top2.contains(&t.g.current(old1)) && top2.contains(&t.g.current(old2)));
    assert_eq!(t.hand_size(P0), 4, "the four drawn stay");
}

#[test]
fn riverwise_augur_draws_and_puts_back_in_one_resolution() {
    cr!("608.2", "117.3b");
    ruling!(
        "Riverwise Augur",
        "You draw three cards and put two cards back all while Riverwise Augur's ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Riverwise Augur");
    t.settle();
    let from = mark(&t);
    t.g.resolve_top();
    assert!(priority_since(&t, from).is_empty());
    assert_eq!(t.hand_size(P0), 1, "three drawn, two put back");
    assert!(choices_since(&t, P0, from).iter().any(|(_, min, max)| (*min, *max) == (2, 2)));
}

// --- Glimpse the Cosmos ----------------------------------------------------------------

const GLIMPSE: &str = "Glimpse the Cosmos";

/// The cast method of Glimpse the Cosmos' own graveyard permission ({U}), if offered.
fn own_method(t: &mut TestGame, card: ObjectId) -> Option<CastMethod> {
    legal_cast_methods(t, P0, card)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
}

#[test]
fn glimpse_the_cosmos_from_the_graveyard_keeps_sorcery_timing_and_mana_value() {
    cr!("601.3", "307.1", "202.3");
    ruling!(
        "Glimpse the Cosmos",
        "Casting Glimpse the Cosmos from your graveyard using the last ability doesn't change when you can cast it."
    );
    ruling!(
        "Glimpse the Cosmos",
        "Glimpse the Cosmos has mana value 2, no matter what you actually paid to cast it."
    );
    supported(GLIMPSE);
    // The opponent's turn: not castable, even with a Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    let g = t.graveyard(P0, GLIMPSE);
    t.set_step(P1, Step::PrecombatMain);
    assert!(own_method(&mut t, g).is_none());
    // Its own main phase with an empty stack: castable for {U}, mana value 2.
    t.set_step(P0, Step::PrecombatMain);
    let m = own_method(&mut t, g).expect("castable from the graveyard");
    let spell = t.cast(P0, g).method(m).go();
    assert_eq!(mana_value(&t, spell), 2);
    // Something on the stack: not castable.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P1, "Mountain", 1);
    let g = t.graveyard(P0, GLIMPSE);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(pl(P1)).go();
    assert!(own_method(&mut t, g).is_none());
    assert!(!can_cast(&mut t, P0, g, CastMethod::Normal));
}

#[test]
fn glimpse_the_cosmos_caster_gets_priority_first_after_it_resolves() {
    cr!("117.3b", "601.3");
    ruling!(
        "Glimpse the Cosmos",
        "If Glimpse the Cosmos resolves during your turn and is put into your graveyard, you receive priority before any other player does."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 3);
    let g = t.hand(P0, GLIMPSE);
    t.cast(P0, g).go();
    let from = mark(&t);
    // Both pass, it resolves, and the game goes on until someone has priority again.
    let ok = t.g.run_until(1000, |g| {
        g.stack.is_empty() && g.turn.priority.is_some() && !g.player(P0).graveyard.is_empty()
    });
    assert!(ok);
    assert_eq!(priority_since(&t, from), vec![P0, P1], "both passed");
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    assert_eq!(t.g.turn.priority, Some(P0), "P0 receives priority first");
    let gy = t.g.player(P0).graveyard.clone();
    let can = t.g.legal_actions(P0).iter().any(|a| {
        matches!(a, Action::Cast { card, .. } if gy.contains(card))
    });
    assert!(can, "P0 may cast it again from the graveyard");
}

#[test]
fn glimpse_the_cosmos_with_another_permission() {
    cr!("601.2f", "702.34a", "601.3");
    ruling!(
        "Glimpse the Cosmos",
        "If you cast Glimpse the Cosmos from your graveyard using a different permission, the last ability won't apply. You'll pay whatever cost that permission stipulates, and you'll follow that permission's instructions regarding exiling Glimpse the Cosmos, if any."
    );
    supported("Snapcaster Mage");
    // No Giant: Snapcaster Mage's flashback costs its mana cost, {1}{U}, and exiles it.
    let mut t = TestGame::new(2);
    let g = t.graveyard(P0, GLIMPSE);
    t.lands(P0, "Island", 2);
    t.answer_targets(P0, &[obj(g)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    let methods = legal_cast_methods(&mut t, P0, g);
    assert!(methods.contains(&CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback)));
    assert!(own_method(&mut t, g).is_none(), "no Giant");
    let spell = t
        .cast(P0, g)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2, "paid {{1}}{{U}}");
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile);
}
