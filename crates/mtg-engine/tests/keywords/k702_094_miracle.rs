//! CR 702.94 Miracle.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::{run_effect, stack_triggers};
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MIRACLE: CastMethod = CastMethod::Keyword(KeywordKind::Miracle);

/// The miracle card in `p`'s hand named `name`.
fn in_hand(t: &TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.g.find_in_zone(Zone::Hand(p), name)[0]
}

#[test]
fn a_miracle_card_revealed_as_the_first_draw_can_be_cast_for_its_miracle_cost() {
    cr!("702.94", "702.94a");
    ruling!(
        "Terminus",
        "Miracle is an alternative cost to cast the spell with miracle."
    );
    assert_supported("Temporal Mastery");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Temporal Mastery");
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(stack_triggers(&t, "Miracle").len(), 1);
    t.resolve();
    // Cast for {1}{U} rather than {5}{U}{U}; its mana value is unchanged.
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(spell).chars.name, "Temporal Mastery");
    assert_eq!(t.g.obj(spell).stack.as_ref().unwrap().cast.method, MIRACLE);
    assert_eq!(t.g.obj(spell).chars.mana_value(), 7);
    assert!(t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Island")
        .all(|o| o.tapped));
    t.resolve();
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert!(t.in_exile("Temporal Mastery"));
}

#[test]
fn only_the_first_card_drawn_in_a_turn_can_be_revealed() {
    cr!("702.94a");
    ruling!(
        "Terminus",
        "Only the first card drawn this way may be revealed and cast using its miracle ability."
    );
    let mut t = TestGame::new(2);
    // The second of three cards drawn at once.
    t.library_top(P0, "Island");
    t.library_top(P0, "Temporal Mastery");
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 3);
    t.settle();
    assert!(stack_triggers(&t, "Miracle").is_empty());
    assert!(t.in_hand(P0, "Temporal Mastery"));
    // Nor a later draw that turn.
    t.library_top(P0, "Temporal Mastery");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(stack_triggers(&t, "Miracle").is_empty());
}

#[test]
fn a_miracle_can_be_cast_on_another_players_turn_ignoring_timing() {
    cr!("702.94a");
    ruling!(
        "Terminus",
        "You can reveal and cast a card with miracle on any turn, not just your own, if it's the first card you've drawn that turn."
    );
    ruling!(
        "Terminus",
        "You cast the card with miracle during the resolution of the triggered ability. Ignore any timing rules based on the card's type."
    );
    assert_supported("Terminus");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PostcombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Terminus");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    // P0 draws during P1's turn (as an effect would make them).
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P1));
    assert!(t.in_graveyard(P0, "Terminus"));
}

#[test]
fn a_miracle_card_that_left_the_hand_cant_be_cast() {
    cr!("702.94a");
    ruling!(
        "Terminus",
        "If the card with miracle leaves your hand before the triggered ability resolves, you won't be able to cast it using its miracle ability."
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Temporal Mastery");
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.settle();
    // In response, the card is discarded.
    let card = in_hand(&t, P0, "Temporal Mastery");
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Graveyard),
        },
        &[Entity::Object(card)],
    );
    t.resolve_all();
    assert!(t.g.extra_turns.is_empty());
    assert!(t.in_graveyard(P0, "Temporal Mastery"));
}

#[test]
fn a_revealed_miracle_card_stays_revealed_until_its_ability_resolves() {
    cr!("702.94b");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Temporal Mastery");
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, false); // don't cast it
    t.g.draw_cards(P0, 1);
    let card = in_hand(&t, P0, "Temporal Mastery");
    // Revealed while the triggered ability waits to be put on the stack...
    assert!(is_revealed(&t.g, card));
    t.settle();
    // ... and while it's on the stack.
    assert_eq!(stack_triggers(&t, "Miracle").len(), 1);
    assert!(is_revealed(&t.g, card));
    t.resolve();
    // Not cast: it stays in hand, no longer revealed.
    assert!(t.stack.is_empty());
    assert_eq!(t.zone(card), Zone::Hand(P0));
    assert!(!is_revealed(&t.g, card));
}

#[test]
fn a_revealed_miracle_card_stops_being_revealed_when_it_leaves_the_hand() {
    cr!("702.94b");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Temporal Mastery");
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.settle();
    let card = in_hand(&t, P0, "Temporal Mastery");
    assert!(is_revealed(&t.g, card));
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::library_top(),
        },
        &[Entity::Object(card)],
    );
    let now = t.g.current(card);
    assert!(!is_revealed(&t.g, now));
    assert!(!is_revealed(&t.g, card));
}

#[test]
fn a_card_not_revealed_isnt_revealed_and_doesnt_trigger() {
    cr!("702.94a", "702.94b");
    ruling!(
        "Terminus",
        "You don't have to reveal a drawn card with miracle if you don't wish to cast it at that time."
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Temporal Mastery");
    t.answer_yes(P0, false);
    t.g.draw_cards(P0, 1);
    t.settle();
    let card = in_hand(&t, P0, "Temporal Mastery");
    assert!(!is_revealed(&t.g, card));
    assert!(stack_triggers(&t, "Miracle").is_empty());
}

#[test]
fn the_miracle_card_is_still_drawn() {
    cr!("702.94a");
    ruling!(
        "Terminus",
        "You still draw the card, whether you use the miracle ability or not. Any ability that triggers whenever you draw a card, for example, will trigger. If you don't cast the card using its miracle ability, it will remain in your hand."
    );
    let mut t = TestGame::new(2);
    // Psychosis Crawler: "Whenever you draw a card, each opponent loses 1 life."
    t.battlefield(P0, "Psychosis Crawler");
    t.library_top(P0, "Temporal Mastery");
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, false); // don't cast
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.in_hand(P0, "Temporal Mastery"));
    assert!(t.g.extra_turns.is_empty());
}

#[test]
fn cost_increases_apply_to_the_miracle_cost() {
    cr!("702.94a", "601.2f");
    ruling!(
        "Terminus",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a miracle cost) you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged"
    );
    let mut t = TestGame::new(2);
    // Thalia: noncreature spells cost {1} more.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.library_top(P0, "Temporal Mastery");
    let islands = t.lands(P0, "Island", 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.resolve();
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(spell).chars.name, "Temporal Mastery");
    assert_eq!(t.g.obj(spell).chars.mana_value(), 7);
    assert!(islands.iter().all(|l| t.g.obj(*l).tapped));
}

#[test]
fn miracle_given_to_cards_in_hand_works_as_they_are_drawn() {
    cr!("702.94a");
    ruling!(
        "Lorehold, the Historian",
        "You can reveal and cast a card with miracle on any turn, not just your own, if it's the first card you've drawn that turn."
    );
    assert_supported("Lorehold, the Historian");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lorehold, the Historian");
    // Lightning Bolt has miracle {2} while it's in P0's hand.
    t.library_top(P0, "Lightning Bolt");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P1, Step::Upkeep);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(stack_triggers(&t, "Miracle").len(), 1);
    t.resolve();
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(spell).stack.as_ref().unwrap().cast.method, MIRACLE);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
