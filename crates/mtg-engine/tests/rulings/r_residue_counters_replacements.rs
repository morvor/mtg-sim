//! Item `rulings-residue-counters` — replacement effects around counters and draws:
//! Archmage Ascension's draw replacement (CR 614.11, 616.1, 121.2), "as this enters"
//! replacements of permanents entering together (CR 614.12, Thief of Blood), effects that
//! add to the counters a permanent is put onto the battlefield with or gets (Prairie Dog,
//! Combine Guildmage, High Score; CR 122.6, 616.1), Spark Rupture's planeswalkers entering
//! as creatures (CR 306.5b, 613), and a copy of a permanent whose "as this enters"
//! choice was never made (CR 707.2, Hollowmurk Siege).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

// --- Archmage Ascension ------------------------------------------------------------------

/// P0's Archmage Ascension with `n` quest counters, and a library of Islands.
fn ascension(t: &mut TestGame, n: u32) -> ObjectId {
    let a = t.battlefield(P0, "Archmage Ascension");
    if n > 0 {
        put(t, a, "quest", n);
    }
    stack_library(t, P0, &["Island"; 8]);
    a
}

/// The number of library searches and shuffles P0 made this turn.
fn searches(t: &TestGame) -> (usize, usize) {
    let s =
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Searched { player: P0, .. }))
            .count();
    let sh =
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Shuffled { player: P0 }))
            .count();
    (s, sh)
}

#[test]
fn archmage_ascension_ignores_cards_put_into_hand_without_drawing() {
    cr!("121.5", "121.6");
    ruling!(
        "Archmage Ascension",
        "If a spell or ability causes you to put cards in your hand without specifically using the word “draw,” Archmage Ascension’s abilities ignore it."
    );
    supported("Archmage Ascension");
    supported("Raise Dead");
    supported("Mulch");
    // With six quest counters, returning a card to hand isn't replaced.
    let mut t = TestGame::new(2);
    let a = ascension(&mut t, 6);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Raise Dead", &[obj(bears)]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(searches(&t).0, 0);
    // Nor are cards put into hand from the library: Mulch puts the four Islands it reveals
    // into P0's hand. Four cards put into hand aren't cards drawn: no quest counter at the
    // end step.
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Mulch", &[]);
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(searches(&t).0, 0);
    t.answer_yes(P0, true);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(a, "quest"), 6);
}

#[test]
fn archmage_ascension_replaces_each_draw_of_several_one_at_a_time() {
    cr!("121.2", "121.6b");
    ruling!(
        "Archmage Ascension",
        "each individual draw may be replaced by Archmage Ascension’s effect. Process the draws one at a time."
    );
    supported("Divination");
    let mut t = TestGame::new(2);
    ascension(&mut t, 6);
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    // Search instead of the first draw, then draw the second normally.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Divination", &[]);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(searches(&t), (1, 1));
    let drew =
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Drew { player: P0, .. }))
            .count();
    assert_eq!(drew, 1);
    // Both replaced: a search and a shuffle for each card.
    let mut t = TestGame::new(2);
    ascension(&mut t, 6);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Divination", &[]);
    assert_eq!(searches(&t), (2, 2));
}

#[test]
fn archmage_ascensions_replacing_one_draw_apply_only_once() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Archmage Ascension",
        "if you control more than one Archmage Ascension with six quest counters on it and you would draw a card, each Archmage Ascension’s replacement effect could apply. Once you use one, the rest are no longer applicable."
    );
    let mut t = TestGame::new(2);
    ascension(&mut t, 6);
    let b = t.battlefield(P0, "Archmage Ascension");
    put(&mut t, b, "quest", 6);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(searches(&t).0, 1);
    // The drawing player chose which one applies.
    let choices = replacement_choices(&t, from);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].0, P0);
}

// --- Thief of Blood ----------------------------------------------------------------------

#[test]
fn thieves_of_blood_entering_together_one_gets_the_counters() {
    cr!("614.12", "614.1c");
    ruling!(
        "Thief of Blood",
        "If more than one Thief of Blood enters the battlefield at the same time, you can apply the replacement effect of any of them."
    );
    // The Thief whose replacement effect applies first removes all three counters and
    // enters with them; the other then removes none and enters with none. (The engine
    // applies them in the order the entering objects are given rather than asking: for
    // identical Thieves, whose timestamp order P0 chooses either way (CR 613.7m), every
    // choice the ruling allows gives the same game.)
    let mut t = TestGame::new(2);
    let bears = with_counters(&mut t, P1, "Grizzly Bears", 3);
    let ids = enter_together(&mut t, &[(P0, "Thief of Blood"), (P0, "Thief of Blood")]);
    let mut got: Vec<u32> = ids.iter().map(|id| plus1(&t, *id)).collect();
    got.sort();
    assert_eq!(got, vec![0, 3]);
    assert_eq!(plus1(&t, bears), 0);
}

// --- Prairie Dog, Combine Guildmage, High Score ------------------------------------------

#[test]
fn prairie_dog_adds_one_to_the_counters_a_creature_enters_with() {
    cr!("614.1c", "122.6");
    ruling!(
        "Prairie Dog",
        "If a creature you control would enter the battlefield with a number of +1/+1 counters on it on a turn where Prairie Dog’s last ability has resolved once, it enters with that many plus one instead."
    );
    supported("Prairie Dog");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Prairie Dog");
    t.lands(P0, "Plains", 5);
    t.activate(P0, dog, 0, &[]).unwrap();
    t.resolve_all();
    let bears = enter_with(&mut t, P0, "Grizzly Bears", 2, false);
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn combine_guildmage_does_nothing_if_either_target_is_illegal() {
    cr!("608.2b", "122.5");
    ruling!(
        "Combine Guildmage",
        "If either target creature is an illegal target by the time Combine Guildmage’s second ability tries to resolve, nothing happens. No creature loses or gets a +1/+1 counter."
    );
    supported("Unsummon");
    for gone in [0, 1] {
        let mut t = TestGame::new(2);
        let a = with_counters(&mut t, P0, "Grizzly Bears", 2);
        let b = t.battlefield(P0, "Hill Giant");
        let target = if gone == 0 { b } else { a };
        let keep = if gone == 0 { a } else { b };
        guildmage_move(&mut t, a, b, |t| {
            cast_new(t, P1, "Unsummon", &[obj(target)]);
            t.resolve();
        });
        let expected = if keep == a { 2 } else { 0 };
        assert_eq!(plus1(&t, keep), expected, "the other target ({gone}) gone");
    }
    // Both legal: the counter moves.
    let mut t = TestGame::new(2);
    let a = with_counters(&mut t, P0, "Grizzly Bears", 2);
    let b = t.battlefield(P0, "Hill Giant");
    guildmage_move(&mut t, a, b, |_| {});
    assert_eq!((plus1(&t, a), plus1(&t, b)), (1, 1));
}

#[test]
fn two_high_scores_each_add_one_counter() {
    cr!("616.1f", "614.1a");
    ruling!(
        "High Score",
        "In the case where you control more than one High Score, each one you control will increase the number of +1/+1 counters placed on a creature you control by one."
    );
    supported("High Score");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "High Score");
    t.battlefield(P0, "High Score");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
}

// --- Spark Rupture -----------------------------------------------------------------------

#[test]
fn spark_rupture_planeswalkers_enter_as_creatures_with_their_loyalty() {
    cr!("306.5b", "613.1d", "613.4b");
    ruling!(
        "Spark Rupture",
        "While Spark Rupture is on the battlefield, planeswalkers will enter the battlefield as creatures, but with the appropriate number of loyalty counters."
    );
    supported("Spark Rupture");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spark Rupture");
    let jace = t.enter(P0, "Jace Beleren");
    t.settle();
    assert!(t.on_battlefield(jace));
    assert!(t.obj_now(jace).is(CardType::Creature));
    assert_eq!(t.counters(jace, "loyalty"), 3);
    assert_eq!(t.pt(jace), (3, 3));
}

// --- Hollowmurk Siege --------------------------------------------------------------------

#[test]
fn hollowmurk_siege_copy_without_a_choice_has_neither_ability() {
    cr!("707.2", "607.2d");
    ruling!(
        "Hollowmurk Siege",
        "If you somehow control Hollowmurk Siege and no choice was made for it (perhaps because another permanent on the battlefield became a copy of it), it has neither of the two abilities."
    );
    supported("Hollowmurk Siege");
    supported("Impossible Man");
    // The real Siege is Sultai: one card for a counter put on a creature each turn.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let siege = t.enter(P0, "Hollowmurk Siege");
    t.settle();
    let man = t.battlefield(P0, "Impossible Man");
    t.lands(P0, "Island", 3);
    t.activate(P0, man, 0, &[obj(siege)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(man).is(CardType::Enchantment));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    give_plus1(&mut t, bears, 1);
    assert_eq!(
        triggers_on_stack(&t, "draw a card"),
        1,
        "only the real Siege's"
    );
    t.resolve_all();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
    // Nor the Abzan ability: attacking triggers neither the real (Sultai) Siege nor the copy.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.obj_now(man).is(CardType::Enchantment), "still a copy");
    assert_eq!(triggers_on_stack(&t, "attacking creature"), 0);
}
