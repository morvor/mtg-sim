//! Rulings batch S33 — putting cards into and onto a library: with only one card to look
//! at, that card is the one put into the hand (CR 609.3); a player putting several cards
//! on top of their library chooses their order, which isn't revealed (CR 401.4, 401.2);
//! "then shuffle and put that card on top" is one action, so a revealed top card isn't
//! the one the shuffle put there (CR 701.24b, 401.5).

use crate::r_s01_common::{supported, stack_library};
use crate::r_s11_common::empty_library;
use crate::r_s25_common::cast_new;
use crate::r_s33_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn sleight_of_hand_with_one_card_in_the_library_it_goes_to_hand() {
    cr!("609.3");
    ruling!(
        "Sleight of Hand",
        "If there is only one card in your library, you put it into your hand."
    );
    supported("Sleight of Hand");
    // "Look at the top two cards of your library. Put one of them into your hand and the
    // other on the bottom of your library."
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Sleight of Hand", &[]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert_eq!(t.library_size(P0), 0);
    assert!(!t.has_lost(P0));
}

#[test]
fn stress_dream_with_one_card_in_the_library_it_goes_to_hand() {
    cr!("609.3");
    ruling!(
        "Stress Dream",
        "If there is only one card in your library, you put it into your hand."
    );
    supported("Stress Dream");
    // "Stress Dream deals 5 damage to up to one target creature. Look at the top two cards
    // of your library. Put one of those cards into your hand and the other on the bottom
    // of your library."
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Stress Dream", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.zone(bears), Zone::Hand(P0));
}

fn is_order(d: &Decision) -> bool {
    matches!(d, Decision::Order { .. })
}

#[test]
fn gravepurge_you_choose_the_order_of_the_cards_put_on_top() {
    cr!("401.4");
    ruling!(
        "Gravepurge",
        "You choose the order of the cards you put on top of your library."
    );
    supported("Gravepurge");
    // "Put any number of target creature cards from your graveyard on top of your library.
    // Draw a card." With either order chosen, the card drawn is the one P0 put on top.
    for top_first in [0usize, 1] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let giant = t.graveyard(P0, "Hill Giant");
        let from = t.asked().len();
        t.answer(
            P0,
            DecisionKind::Order,
            Answer::Indices(if top_first == 0 { vec![0, 1] } else { vec![1, 0] }),
        );
        cast_one_slot(
            &mut t,
            P0,
            "Gravepurge",
            &[Entity::Object(bears), Entity::Object(giant)],
        );
        t.resolve_all();
        let orders: Vec<Vec<String>> = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && is_order(d))
            .map(|(_, d)| match d {
                Decision::Order { items, .. } => items.clone(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(orders.len(), 1, "P0 chooses the order once");
        // The cards are listed in the order they're put there: the last one ends up on
        // top.
        let items = &orders[0];
        let drawn = if top_first == 0 { &items[1] } else { &items[0] };
        let other = if top_first == 0 { &items[0] } else { &items[1] };
        let name = |s: &String| {
            if s.contains("Grizzly Bears") {
                "Grizzly Bears"
            } else {
                "Hill Giant"
            }
        };
        assert!(t.in_hand(P0, name(drawn)), "{items:?} {top_first}");
        assert!(!t.in_hand(P0, name(other)));
        let top = t.g.library_top(P0).unwrap();
        assert_eq!(t.g.obj(top).chars.name, name(other));
    }
}

#[test]
fn reinforcements_the_cards_are_shown_but_not_their_order() {
    cr!("401.4", "401.2", "115.1");
    ruling!(
        "Reinforcements",
        "You have to show which creature cards you put on top of your library, but not the order you put them there."
    );
    supported("Reinforcements");
    // "Put up to three target creature cards from your graveyard on top of your library."
    let mut t = TestGame::new(2);
    let cards = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Hill Giant"),
        t.graveyard(P0, "Craw Wurm"),
    ];
    let targets: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 0, 1]));
    let spell = cast_one_slot(&mut t, P0, "Reinforcements", &targets);
    // Which cards: the targets, public while the spell is on the stack.
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), targets);
    t.resolve_all();
    // P0 chose the order (no one else was asked).
    let asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_order(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P0]);
    // The cards are in P0's library, hidden: P1 can't see which one is where.
    let top3 = &library_top_first(&t, P0)[..3];
    let mut names: Vec<String> = top3
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    for c in top3 {
        assert!(!can_look_at(&t.g, P1, *c));
    }
    names.sort();
    assert_eq!(names, vec!["Craw Wurm", "Grizzly Bears", "Hill Giant"]);
}

/// P0, with Goblin Spy ("Play with the top card of your library revealed."), casts the
/// tutor `name` for Hill Giant (instant or not, the tutored card is known) in a library
/// of distinct cards. Returns the cards revealed on top of P0's library while it
/// resolved, and the tutored card.
fn tutor_with_revealed_top(name: &str, card: &str) -> (Vec<ObjectId>, ObjectId) {
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let wanted = t.library_top(P0, card);
    stack_library(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Craw Wurm",
            "Forest",
            "Island",
            "Swamp",
            "Mountain",
            "Plains",
            "Wastes",
        ],
    );
    t.battlefield(P0, "Goblin Spy");
    t.g.recompute();
    t.answer_choose(P0, &[Entity::Object(wanted)]);
    cast_new(&mut t, P0, name, &[]);
    let from = t.g.turn_events.len();
    t.resolve_all();
    let revealed: Vec<ObjectId> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Custom {
                name,
                player: Some(p),
                obj: Some(o),
                ..
            } if name == mtg_engine::zones::TOP_REVEALED && *p == P0 => Some(*o),
            _ => None,
        })
        .collect();
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(t.g.current(wanted), top);
    (revealed, top)
}

#[test]
fn vampiric_tutor_shuffle_and_put_on_top_is_one_action() {
    cr!("701.24b", "401.5");
    ruling!(
        "Vampiric Tutor",
        "The \"shuffle and put the card on top\" is a single action. If an effect causes the top card of the library to be face up, the second card down is not revealed."
    );
    supported("Vampiric Tutor");
    // "Search your library for a card, then shuffle and put that card on top."
    for _ in 0..5 {
        let (revealed, top) = tutor_with_revealed_top("Vampiric Tutor", "Hill Giant");
        assert_eq!(revealed, vec![top]);
    }
}

#[test]
fn mystical_tutor_shuffle_and_put_on_top_is_one_action() {
    cr!("701.24b", "401.5");
    ruling!(
        "Mystical Tutor",
        "The \"shuffle and put the card on top\" is a single action. If an effect causes the top card of the library to be face up, the second card down is not revealed."
    );
    supported("Mystical Tutor");
    for _ in 0..5 {
        let (revealed, top) = tutor_with_revealed_top("Mystical Tutor", "Divination");
        assert_eq!(revealed, vec![top]);
    }
}
