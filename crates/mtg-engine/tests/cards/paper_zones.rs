//! Zone phrases of older Oracle wordings (cards4-paper-effects, family `zone-phrases`):
//! "shuffle ~ into your library" from the graveyard, "the top card of your graveyard",
//! "reveal your hand and put all land cards from it onto the battlefield".

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn guan_yu_shuffles_itself_from_the_graveyard() {
    cr!("400.7", "603.10a", "701.24a");
    supported("Guan Yu, Sainted Warrior");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let g = t.battlefield(P0, "Guan Yu, Sainted Warrior");
    let lib = t.library_size(P0);
    t.answer_yes(P0, true);
    t.g.destroy(g, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Guan Yu, Sainted Warrior"));
    assert_eq!(t.library_size(P0), lib + 1);
}

#[test]
fn guan_yu_is_not_shuffled_if_it_left_the_graveyard() {
    cr!("400.7");
    ruling!(
        "Guan Yu, Sainted Warrior",
        "If Guan Yu, Sainted Warrior is removed from the graveyard after the ability triggers but before it resolves, it won't get shuffled into your library."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let g = t.battlefield(P0, "Guan Yu, Sainted Warrior");
    let lib = t.library_size(P0);
    t.answer_yes(P0, true);
    t.g.destroy(g, None);
    t.g.flush_events();
    t.settle();
    // In response, the card is exiled from the graveyard.
    let card = t.g.current(g);
    t.g.exile_object(card, None);
    t.resolve_all();
    assert!(t.in_exile("Guan Yu, Sainted Warrior"));
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn soldevi_digger_moves_the_top_card_of_the_graveyard_to_the_bottom() {
    cr!("404.1", "404.2");
    supported("Soldevi Digger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let d = t.battlefield(P0, "Soldevi Digger");
    t.lands(P0, "Plains", 2);
    t.library_top(P0, "Island");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve();
    // Hill Giant was put into the graveyard last: it's the top card.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.g.obj(bottom).chars.name, "Hill Giant");
}

#[test]
fn manabond_puts_lands_onto_the_battlefield_and_discards_the_rest() {
    cr!("701.20a", "701.9a");
    ruling!(
        "Manabond",
        "You discard all the cards still in your hand after you put the lands onto the battlefield."
    );
    supported("Manabond");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manabond");
    t.hand(P0, "Forest");
    t.hand(P0, "Island");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn manabond_discards_even_without_lands_but_not_if_declined() {
    cr!("701.9a");
    // Revealing a hand with no land: "if you do" is the reveal, so the hand is discarded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manabond");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Declining: nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manabond");
    t.hand(P0, "Forest");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Forest").is_empty());
}
