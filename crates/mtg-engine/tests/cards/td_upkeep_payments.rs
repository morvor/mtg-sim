//! Triggers on paying (or not paying) an upkeep cost: "When ~'s echo cost is paid" (Shah
//! of Naar Isle), "Whenever you pay ~'s cumulative upkeep" (Hibernation's End), "When a
//! player doesn't pay ~'s cumulative upkeep" (Heart of Bogardan).

use crate::combat2_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

#[test]
fn shah_of_naar_isle_paying_echo_lets_each_opponent_draw_up_to_three() {
    cr!("702.30a", "603.2");
    ruling!(
        "Shah of Naar Isle",
        "The first option means Shah of Naar Isle stays on the battlefield and its last ability triggers."
    );
    supported("Shah of Naar Isle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shah of Naar Isle");
    next_upkeep(&mut t, P0);
    let hand = t.hand_size(P1);
    t.answer_yes(P0, true);
    t.answer(P1, DecisionKind::Number, Answer::Number(2));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Shah of Naar Isle").len(), 1);
    assert_eq!(t.hand_size(P1), hand + 2);
}

#[test]
fn shah_of_naar_isle_not_paying_echo_doesnt_trigger() {
    cr!("702.30a");
    let mut t = TestGame::new(2);
    let shah = t.battlefield(P0, "Shah of Naar Isle");
    next_upkeep(&mut t, P0);
    let hand = t.hand_size(P1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(shah));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn hibernations_end_searches_for_mana_value_equal_to_age_counters() {
    cr!("702.24a", "603.2");
    ruling!(
        "Hibernation's End",
        "must be exactly equal to the number of age counters"
    );
    supported("Hibernation's End");
    let mut t = TestGame::new(2);
    let he = t.battlefield(P0, "Hibernation's End");
    t.lands(P0, "Forest", 3);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Llanowar Elves");
    // First upkeep: one age counter; pay {1}: a creature with mana value 1.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(he, "age"), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // Second upkeep: two; pay {2}: mana value 2.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn hibernations_end_not_paying_doesnt_trigger() {
    cr!("702.24a");
    let mut t = TestGame::new(2);
    let he = t.battlefield(P0, "Hibernation's End");
    t.library_top(P0, "Llanowar Elves");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(he));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
}

#[test]
fn heart_of_bogardan_deals_damage_when_its_upkeep_isnt_paid() {
    cr!("702.24a", "603.10a");
    ruling!(
        "Heart of Bogardan",
        "X is equal to the amount of mana last spent to pay its cumulative upkeep"
    );
    supported("Heart of Bogardan");
    let mut t = TestGame::new(2);
    let heart = t.battlefield(P0, "Heart of Bogardan");
    t.lands(P0, "Mountain", 2);
    let ogre = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // First upkeep: paid; nothing.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(heart));
    assert_eq!(t.life(P1), 20);
    // Second upkeep: two age counters, not paid: X = 2*2-2 = 2 to P1 and each of P1's
    // creatures.
    next_upkeep(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(heart));
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(ogre));
    assert_eq!(t.obj_now(ogre).damage, 2);
}
