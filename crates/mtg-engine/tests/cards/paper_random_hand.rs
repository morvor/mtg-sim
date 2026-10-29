//! Random hand cards in older Oracle wordings (cards4-paper-effects, family
//! `random-hand`): "Look at a card at random in target player's hand", "reveals X cards
//! at random", "Discard any number of cards at random".

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn urzas_bauble_looks_privately_at_a_random_card() {
    cr!("402.3", "701.20e", "400.7");
    interp!("looked-at-hand-card-stays-seen");
    supported("Urza's Bauble");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let bauble = t.battlefield(P0, "Urza's Bauble");
    let card = t.hand(P1, "Lightning Bolt");
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, card));
    t.activate(P0, bauble, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    // The controller now sees it; it isn't revealed to anyone else.
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, card));
    assert!(!mtg_engine::reveal::is_revealed(&t.g, card));
    // Only the looking player: a third player still can't see it.
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P2, card));
    assert!(t.in_hand(P1, "Lightning Bolt"));
    let hand = t.hand_size(P0);
    // "You draw a card at the beginning of the next turn's upkeep."
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, card));
    // Once it changes zones it's a new object the looker hasn't seen (CR 400.7): put on
    // top of its owner's library, it's hidden again.
    let moved = t
        .g
        .move_object(card, mtg_engine::object::Zone::Library(P1), events::MoveCause::Effect, None)
        .expect("moved");
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, moved));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, card));
}

#[test]
fn nebuchadnezzar_discards_only_revealed_cards_with_the_name() {
    cr!("701.20a", "701.9a");
    ruling!(
        "Nebuchadnezzar",
        "You don’t name a card until the ability resolves."
    );
    supported("Nebuchadnezzar");
    // X = 3 reveals the whole hand: both Bears are discarded, the Giant stays.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let n = t.battlefield(P0, "Nebuchadnezzar");
    t.lands(P0, "Island", 3);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.activate(P0, n, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(!t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(
        t.g.player(P1)
            .graveyard
            .iter()
            .filter(|o| t.g.obj(**o).chars.name == "Grizzly Bears")
            .count(),
        2
    );
    // X = 1 with two Bears in hand: only the revealed one is discarded.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let n = t.battlefield(P0, "Nebuchadnezzar");
    t.lands(P0, "Island", 1);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.activate(P0, n, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn rites_of_initiation_discards_the_chosen_number_at_random() {
    cr!("701.9b", "608.2d");
    supported("Rites of Initiation");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let rites = t.hand(P0, "Rites of Initiation");
    t.lands(P0, "Mountain", 1);
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.cast(P0, rites).go();
    let before = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), before - 2);
    assert_eq!(t.pt(bears), (4, 2));
}
