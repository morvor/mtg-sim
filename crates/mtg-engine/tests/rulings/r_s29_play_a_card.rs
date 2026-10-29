//! Rulings batch S29 — "Whenever you play a card" (Recycle, Null Profusion, Juju Bubble):
//! to play a card is to play it as a land or cast it as a spell (glossary "Play"); the
//! ability triggers on casting (CR 601.2i) and resolves even if the spell is countered;
//! and maximum hand size (CR 402.2, 514.1, 613.7).

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn countering_the_spell_doesnt_stop_the_draw() {
    cr!("601.2i", "603.3", "701.6a");
    ruling!(
        "Recycle",
        "Countering a spell that has been cast will not prevent you from drawing the card."
    );
    supported("Recycle");
    supported("Null Profusion");
    // "Whenever you play a card, draw a card."
    for card in ["Recycle", "Null Profusion"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let hand = t.hand_size(P0);
        let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{card} triggered");
        cast_new(&mut t, P1, "Counterspell", &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        assert_eq!(t.hand_size(P0), hand + 1, "{card}");
        // Playing a land is playing a card too.
        let forest = t.hand(P0, "Forest");
        let hand = t.hand_size(P0);
        t.play_land(P0, forest).unwrap();
        t.settle();
        t.resolve_all();
        // The land left the hand, and a card was drawn for it.
        assert_eq!(t.hand_size(P0), hand, "{card}: drew for the land too");
    }
}

#[test]
fn casting_a_copy_of_a_card_isnt_playing_a_card() {
    cr!("707.12", "601.2i");
    ruling!(
        "Null Profusion",
        "The triggered ability will trigger when you play a land card or cast a nonland card as a spell. It won't trigger when you play a copy of a card, such as with Isochron Scepter."
    );
    ruling!(
        "Recycle",
        "To play a card means to play a land or to cast a spell that's a card (and not a copy of a card)."
    );
    supported("Isochron Scepter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Null Profusion");
    // Isochron Scepter exiles Lightning Bolt, then P0 casts a copy of it.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    let hand = t.hand_size(P0);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s06_common::activate_containing(&mut t, P0, scepter, "copy").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the copy was cast");
    assert_eq!(t.hand_size(P0), hand, "casting the copy didn't trigger");
}

#[test]
fn a_card_played_from_anywhere_counts() {
    cr!("601.2i", "702.34a");
    ruling!(
        "Juju Bubble",
        "It does not matter if the card is played from your hand or from somewhere else."
    );
    supported("Juju Bubble");
    supported("Think Twice");
    // Juju Bubble: "When you play a card, sacrifice ~." Think Twice cast with flashback
    // from the graveyard.
    let mut t = TestGame::new(2);
    let bubble = t.battlefield(P0, "Juju Bubble");
    let think = t.graveyard(P0, "Think Twice");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, think)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(bubble));
    assert!(t.in_graveyard(P0, "Juju Bubble"));
}

#[test]
fn maximum_hand_size_is_checked_only_in_the_cleanup_step() {
    cr!("402.2", "514.1");
    ruling!(
        "Recycle",
        "Your maximum hand size is checked only during the cleanup step of your turn. At any other time, you may have any number of cards in hand."
    );
    // Recycle: "Your maximum hand size is two." P0 holds five cards all turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Recycle");
    for _ in 0..5 {
        t.hand(P0, "Grizzly Bears");
    }
    t.advance_to(P0, Step::End);
    assert_eq!(t.hand_size(P0), 5);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 2);
    // P1's cleanup step doesn't check P0's hand.
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 5);
}

#[test]
fn hand_size_effects_apply_in_timestamp_order() {
    cr!("402.2", "613.7");
    ruling!(
        "Recycle",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Spellbook (an artifact that says you have no maximum hand size) onto the battlefield and then put Recycle onto the battlefield, your maximum hand size will be two. However, if those permanents entered in the opposite order, you would have no maximum hand size."
    );
    supported("Spellbook");
    for (first, second, expected) in [
        ("Spellbook", "Recycle", Some(2)),
        ("Recycle", "Spellbook", None),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, first);
        t.battlefield(P0, second);
        t.g.recompute();
        assert_eq!(
            t.g.player(P0).max_hand_size,
            expected,
            "{first} then {second}"
        );
    }
}
