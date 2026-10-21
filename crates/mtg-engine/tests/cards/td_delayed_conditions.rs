//! Delayed triggered abilities with conditions about the cards they return (CR 603.4,
//! 603.7c): "if you haven't cast the card", "if you haven't played it", "any of those
//! cards you didn't play" (Psychic Theft, Planeswalker's Mischief, Grinning Totem, Three
//! Wishes).

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Psychic Theft on P1, whose hand is a Lightning Bolt (taken).
fn theft(t: &mut TestGame) -> ObjectId {
    let bolt = t.hand(P1, "Lightning Bolt");
    let theft = t.hand(P0, "Psychic Theft");
    mana(t, P0);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast_with(P0, theft, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let exiled = t.g.current(bolt);
    assert_eq!(t.zone(exiled), Zone::Exile);
    exiled
}

#[test]
fn psychic_theft_returns_the_card_if_you_didnt_cast_it() {
    cr!("603.4", "603.7a");
    supported("Psychic Theft");
    let mut t = TestGame::new(2);
    let bolt = theft(&mut t);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Hand(P1));
}

#[test]
fn psychic_theft_cast_before_the_end_step_doesnt_trigger() {
    cr!("603.4");
    interp!("delayed-if-not-played-means-still-exiled");
    let mut t = TestGame::new(2);
    let bolt = theft(&mut t);
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn psychic_theft_cast_in_response_to_the_trigger() {
    cr!("603.4");
    ruling!(
        "Psychic Theft",
        "as long as you do so before that trigger resolves"
    );
    let mut t = TestGame::new(2);
    let bolt = theft(&mut t);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_hand(P1, "Lightning Bolt"));
}

#[test]
fn grinning_totem_card_goes_to_the_graveyard_at_your_next_upkeep_if_not_played() {
    cr!("603.4", "603.7a");
    ruling!(
        "Grinning Totem",
        "the effect wears off as soon as your next upkeep starts"
    );
    supported("Grinning Totem");
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Grinning Totem");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.lands(P0, "Island", 2);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, totem, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let exiled = t.g.current(bolt);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn three_wishes_puts_the_unplayed_cards_into_your_graveyard() {
    cr!("603.7a", "603.7c");
    ruling!(
        "Three Wishes",
        "Between the start of your upkeep and putting the extra cards into the graveyard, you can’t play them."
    );
    supported("Three Wishes");
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Lightning Bolt");
    let b = t.library_top(P0, "Grizzly Bears");
    let c = t.library_top(P0, "Forest");
    let wishes = t.hand(P0, "Three Wishes");
    mana(&mut t, P0);
    t.cast_with(P0, wishes, &[]).unwrap();
    t.resolve_all();
    for x in [a, b, c] {
        assert_eq!(t.zone(t.g.current(x)), Zone::Exile);
    }
    // Play the land.
    t.play_land(P0, t.g.current(c)).unwrap();
    assert!(t.on_battlefield(t.g.current(c)));
    // Still exiled through the opponent's turn.
    t.advance_to(P1, Step::End);
    assert_eq!(t.zone(t.g.current(a)), Zone::Exile);
    // P0's next upkeep.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(t.g.current(c)));
}
