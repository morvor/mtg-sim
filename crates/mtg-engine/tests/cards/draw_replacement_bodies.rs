//! Draw replacements with several instructions (CR 614.6, 614.11, 614.5): "reveal the top
//! card of your library instead. If it's a creature card, put it into your graveyard.
//! Otherwise, draw a card." (the draw the replacement performs isn't replaced again),
//! "instead they draw a card and reveal it. If it's a creature card, that player discards
//! it unless they pay 3 life.", and "the next time they would draw a card this turn,
//! instead they exile the top card of their library. They may play it this turn."

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn enduring_renewal_mills_creatures_and_draws_other_cards() {
    cr!("614.6", "614.5", "121.1");
    ruling!("Enduring Renewal", "the card you draw as a result of the second ability will be the card you revealed");
    assert_supported("Enduring Renewal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Renewal");
    t.library_top(P0, "Island");
    t.library_top(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    // The Bears is revealed and put into the graveyard: nothing drawn.
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
    // The Island is revealed, then drawn.
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn breathstealers_crypt_creature_drawn_is_discarded_unless_life_is_paid() {
    cr!("614.6", "118.12a");
    assert_supported("Breathstealer's Crypt");
    // P1 doesn't pay: the creature card drawn is discarded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Breathstealer's Crypt");
    t.library_top(P1, "Grizzly Bears");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    // P1 pays 3 life: kept.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Breathstealer's Crypt");
    t.library_top(P1, "Grizzly Bears");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    // A noncreature card is just drawn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Breathstealer's Crypt");
    t.library_top(P1, "Island");
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert!(t.in_hand(P1, "Island"));
}

#[test]
fn urabrask_exiles_the_opponents_next_draw_and_they_may_play_it() {
    cr!("614.6", "611.2c");
    ruling!("Urabrask, Heretic Praetor", "Players must pay all costs and follow all normal timing rules");
    assert_supported("Urabrask, Heretic Praetor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urabrask, Heretic Praetor");
    let top = t.library_top(P1, "Forest");
    let hand = t.hand_size(P1);
    t.advance_to(P1, Step::PrecombatMain);
    // The draw step's draw was replaced: the Forest was exiled instead.
    assert_eq!(t.hand_size(P1), hand);
    let forest = t.g.current(top);
    assert!(t.in_exile("Forest"));
    // P1 may play it this turn (as a land, following the normal rules).
    t.play_land(P1, forest).unwrap();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}
