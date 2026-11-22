//! CR 701.29: fateseal, and CR 701.30: clash.

use crate::a701_028_071_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::fateseal_clash::CLASHED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn fateseal_arranges_the_top_cards_of_an_opponents_library() {
    cr!("701.29a");
    ruling!(
        "Spin into Myth",
        "The opponent chosen for the fateseal action doesn’t have to be the player who controlled the target creature."
    );
    supported("Spin into Myth");
    // "Put target creature on top of its owner's library, then fateseal 2."
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P1, "Hill Giant");
    let a = t.library_top(P2, "Grizzly Bears");
    let b = t.library_top(P2, "Llanowar Elves");
    let c = t.library_top(P2, "Forest");
    t.lands(P0, "Island", 5);
    let spell = t.hand(P0, "Spin into Myth");
    t.cast(P0, spell).target(giant).go();
    // Fateseal P2's library: the Forest to the bottom, the Elves stay on top.
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Player(P2)]));
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![b], vec![c]));
    t.resolve_all();
    assert_eq!(t.g.library_top(P1).map(|o| t.obj(o).chars.name.clone()).as_deref(), Some("Hill Giant"));
    let lib = &t.g.player(P2).library;
    assert_eq!(lib[0], c);
    assert_eq!(*lib.last().unwrap(), b);
    assert_eq!(lib[lib.len() - 2], a);
    // P0 looked at the top two cards of that library.
    let looked: Vec<Vec<ObjectId>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards } if p == P0 => Some(cards),
            _ => None,
        })
        .collect();
    assert_eq!(looked, vec![vec![c, b]]);
}

#[test]
fn clash_reveal_top_cards_and_the_higher_mana_value_wins() {
    cr!("701.30a", "701.30b", "701.30d");
    supported("Adder-Staff Boggart");
    // "When this creature enters, clash with an opponent. If you win, put a +1/+1 counter
    // on this creature."
    let mut t = TestGame::new(2);
    let mine = t.library_top(P0, "Hill Giant");
    let theirs = t.library_top(P1, "Grizzly Bears");
    // P0 puts theirs on the bottom; P1 keeps theirs on top.
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    let boggart = t.enter(P0, "Adder-Staff Boggart");
    t.resolve_all();
    assert_eq!(t.counters(boggart, "+1/+1"), 1);
    assert_eq!(t.g.player(P0).library[0], mine);
    assert_eq!(t.g.library_top(P1), Some(theirs));
    assert_eq!(
        custom_events(&t, CLASHED),
        vec![(Some(P0), None, 1), (Some(P1), None, 0)]
    );
    // A tie: nobody wins.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Runeclaw Bear");
    let boggart = t.enter(P0, "Adder-Staff Boggart");
    t.resolve_all();
    assert_eq!(t.counters(boggart, "+1/+1"), 0);
    assert_eq!(
        custom_events(&t, CLASHED),
        vec![(Some(P0), None, 0), (Some(P1), None, 0)]
    );
    // A higher mana value for the opponent: they win, not you.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    let boggart = t.enter(P0, "Adder-Staff Boggart");
    t.resolve_all();
    assert_eq!(t.counters(boggart, "+1/+1"), 0);
    assert_eq!(custom_events(&t, CLASHED)[1], (Some(P1), None, 1));
}

#[test]
fn clashing_players_decide_in_apnap_order_after_revealing() {
    cr!("701.30c");
    let mut t = TestGame::new(2);
    let mine = t.library_top(P0, "Grizzly Bears");
    let theirs = t.library_top(P1, "Hill Giant");
    // It's P1's turn: P1 decides first, then P0.
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.enter(P0, "Adder-Staff Boggart");
    t.resolve_all();
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| matches!(d, Decision::YesNo { .. }).then_some(p))
        .collect();
    assert_eq!(order, vec![P1, P0]);
    // Both cards moved.
    assert_eq!(t.g.player(P0).library[0], mine);
    assert_eq!(t.g.player(P1).library[0], theirs);
    assert_eq!(t.zone(mine), Zone::Library(P0));
}

#[test]
fn a_player_clashes_even_when_an_opponent_initiated_it() {
    cr!("701.30b", "701.30d");
    ruling!(
        "Rebellion of the Flamekin",
        "If you clash because of a spell or ability an opponent controls, the ability will still trigger. Likewise, you can still win the clash even if you weren't the player to initiate it."
    );
    supported("Rebellion of the Flamekin");
    // "Whenever you clash, you may pay {1}. If you do, create a 3/1 red Elemental Shaman
    // creature token. If you won, that token gains haste until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rebellion of the Flamekin");
    t.lands(P1, "Mountain", 1);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    t.answer_yes(P1, false); // keep the revealed card on top
    t.answer_yes(P1, true); // pay {1}
    t.enter(P0, "Adder-Staff Boggart");
    t.resolve_all();
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.controller == P1)
        .map(|o| o.id)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (3, 1));
    // P1 won the clash P0 started: the token has haste.
    assert!(t.obj(tokens[0]).has_keyword(KeywordKind::Haste));
}
