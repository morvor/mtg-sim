//! Rulings batch S10 — madness (CR 702.35): "If you discard this card, discard it into
//! exile. When you do, cast it for its madness cost or put it into your graveyard."

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s04_common::add_mana;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn milling_a_madness_card_isnt_discarding_it() {
    cr!("701.9a", "702.35a", "701.17a");
    ruling!(
        "Markov Baron",
        "Cards are discarded in a Magic game only from a player's hand. Effects that put cards into a player's graveyard from anywhere else do not cause those cards to be discarded."
    );
    supported("Markov Baron");
    supported("Thought Scour");
    // Thought Scour: "Target player mills two cards. Draw a card." Markov Baron (madness
    // {2}{B}) is milled: it goes to the graveyard, not into exile, and madness doesn't
    // trigger.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Markov Baron");
    t.library_top(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::U, 1);
    let scour = t.hand(P0, "Thought Scour");
    t.cast(P0, scour).target(P0).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Markov Baron"));
    assert!(!t.in_exile("Markov Baron"));
    assert_eq!(triggers_on_stack(&t, "Madness"), 0);
    assert_eq!(t.stack_len(), 0);
    // Discarded from the hand, it's exiled and madness triggers.
    let baron = t.hand(P0, "Markov Baron");
    t.g.discard(P0, baron, None);
    t.g.flush_events();
    t.settle();
    assert!(t.in_exile("Markov Baron"));
    assert_eq!(triggers_on_stack(&t, "Madness"), 1);
}

#[test]
fn a_creature_with_madness_can_be_cast_during_an_opponents_turn() {
    cr!("702.35a", "702.35b", "302.1");
    ruling!(
        "Markov Baron",
        "Casting a spell with madness ignores the timing rules based on the card's card type. For example, you can cast a creature with madness if you discard it during an opponent's turn."
    );
    supported("Mind Rot");
    // During P1's turn, P1's Mind Rot makes P0 discard Markov Baron.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Swamp", 3);
    t.hand(P0, "Markov Baron");
    t.hand(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.cast(P1, rot).target(P0).go();
    t.resolve();
    assert!(t.in_exile("Markov Baron"));
    assert_eq!(triggers_on_stack(&t, "Madness"), 1);
    // P0 casts it for {2}{B}; it resolves during P1's turn.
    t.answer_yes(P0, true);
    t.resolve();
    let baron = t.g.find_in_zone(Zone::Stack, "Markov Baron");
    assert_eq!(baron.len(), 1);
    t.resolve_all();
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.named_on_battlefield("Markov Baron").len(), 1);
}
