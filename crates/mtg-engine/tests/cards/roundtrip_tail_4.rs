//! roundtrip-tail-4 (docs/agents/items/roundtrip-tail-4.md): parser bugs the round trip
//! found in its clusters, and cards whose rendering differs in wording, checked in play.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn discerning_taste_counts_the_cards_put_into_the_graveyard_not_the_one_taken() {
    cr!("608.2c", "400.7");
    // "Put one of them into your hand and the rest into your graveyard. You gain life
    // equal to the greatest power among creature cards put into your graveyard this way."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Grizzly Bears");
    let wurm = t.library_top(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Discerning Taste");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The Bears' power (2), not the Wurm's (6).
    assert_eq!(t.life(P0), 22, "{}", t.dump_log());
}

#[test]
fn milled_cards_exiled_instead_were_not_put_into_the_graveyard_this_way() {
    cr!("608.2c", "614.1a", "701.17c");
    // Liliana's Indignation: "Mill X cards. Target player loses 2 life for each creature
    // card put into your graveyard this way." With Rest in Peace, the milled cards are
    // exiled instead: none was put into the graveyard.
    let mut t = TestGame::new(2);
    t.enter(P0, "Rest in Peace");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Liliana's Indignation");
    t.cast(P0, spell).x(2).target(P1).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
}

#[test]
fn milled_cards_put_into_the_graveyard_still_count() {
    cr!("608.2c");
    // (The same without Rest in Peace: two creature cards, 4 life.)
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Liliana's Indignation");
    t.cast(P0, spell).x(2).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
}

#[test]
fn coerced_confession_draws_nothing_for_cards_exiled_instead() {
    cr!("608.2c", "614.1a");
    // "Target player mills four cards. You draw a card for each creature card put into
    // their graveyard this way." Leyline of the Void exiles the opponent's milled cards.
    let mut t = TestGame::new(2);
    t.enter(P0, "Leyline of the Void");
    for _ in 0..4 {
        t.library_top(P1, "Grizzly Bears");
    }
    for _ in 0..4 {
        t.library_top(P0, "Forest");
    }
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Coerced Confession");
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 0, "{}", t.dump_log());
}

#[test]
fn king_narfis_betrayal_exiles_a_card_from_each_graveyard() {
    cr!("714.2b", "608.2c");
    // "I — Each player mills four cards. Then you may exile a creature or planeswalker card
    // from each graveyard."
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        for _ in 0..4 {
            t.library_top(p, "Forest");
        }
    }
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.enter(P0, "King Narfi's Betrayal");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
    let _ = Zone::Battlefield;
}
