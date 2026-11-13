//! CR 701.17: mill.

use crate::a701_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn milling_puts_cards_from_the_top_of_the_library_into_the_graveyard() {
    cr!("701.17", "701.17a");
    supported("Thought Scour");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let a = t.library_top(P1, "Grizzly Bears");
    let b = t.library_top(P1, "Hill Giant");
    let scour = t.hand(P0, "Thought Scour");
    t.cast(P0, scour).target(P1).go();
    t.resolve();
    assert!(!t.g.is_live(a) && !t.g.is_live(b));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn a_player_mills_as_many_cards_as_possible() {
    cr!("701.17b");
    supported("Thought Scour");
    let mut t = TestGame::new(2);
    clear_library(&mut t, P1);
    t.library_top(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let scour = t.hand(P0, "Thought Scour");
    t.cast(P0, scour).target(P1).go();
    t.resolve();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 1);
    assert!(!t.has_lost(P1));
}

#[test]
fn a_player_cant_choose_to_mill_more_cards_than_their_library_has() {
    cr!("701.17b");
    let def = oracle_card(
        "Dig Deep",
        "Sorcery",
        "{0}",
        None,
        "You may mill two cards. If you do, you gain 3 life.",
    );
    let mut t = TestGame::new(2);
    clear_library(&mut t, P0);
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    let s = t.custom(P0, def.clone(), Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    // The choice couldn't be made: nothing was milled and no life was gained.
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.life(P0), 20);
    // With two cards, it can.
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.life(P0), 23);
    // Nor can a mill cost that's too large be paid.
    let cost = oracle_card(
        "Grave Tithe",
        "Artifact",
        "{0}",
        None,
        "Mill two cards: You gain 1 life.",
    );
    let mut t = TestGame::new(2);
    clear_library(&mut t, P0);
    t.library_top(P0, "Grizzly Bears");
    let a = t.custom(P0, cost, Zone::Battlefield);
    assert!(t.activate(P0, a, 0, &[]).is_err());
}

#[test]
fn a_milled_card_is_found_where_it_went() {
    cr!("701.17c");
    supported("Leyline Dowser");
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    // "If a card or token would be put into a graveyard from anywhere, exile it instead."
    t.battlefield(P1, "Rest in Peace");
    t.lands(P0, "Island", 1);
    let dowser = t.battlefield(P0, "Leyline Dowser");
    let bolt = t.library_top(P0, "Lightning Bolt");
    // "{1}, {T}: Mill a card. You may put an instant or sorcery card milled this way into
    // your hand."
    t.answer_yes(P0, true);
    let i = 0;
    t.activate(P0, dowser, i, &[]).unwrap();
    t.resolve();
    // The Bolt was milled into exile, and found there.
    assert!(!t.g.is_live(bolt));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.in_exile("Lightning Bolt"));
}

#[test]
fn information_about_the_milled_card_comes_from_each_card_milled() {
    cr!("701.17d");
    supported("Mindshrieker");
    supported("Bruvac the Grandiloquent");
    let mut t = TestGame::new(2);
    // "If an opponent would mill one or more cards, they mill twice that many cards
    // instead."
    t.battlefield(P0, "Bruvac the Grandiloquent");
    let shrieker = t.battlefield(P0, "Mindshrieker");
    t.library_top(P1, "Hill Giant"); // mana value 4
    t.library_top(P1, "Grizzly Bears"); // mana value 2
    t.lands(P0, "Island", 2);
    // "{2}: Target player mills a card. This creature gets +X/+X until end of turn, where
    // X is the milled card's mana value."
    t.activate(P0, shrieker, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.pt(shrieker), (1 + 6, 1 + 6));
    // Its controller isn't an opponent: only one card.
    t.lands(P0, "Island", 2);
    t.library_top(P0, "Grizzly Bears");
    t.activate(P0, shrieker, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 1);
}
