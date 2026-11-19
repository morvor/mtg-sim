//! Item `rulings-residue-counters` — replacement effects around counters and draws:
//! Archmage Ascension's draw replacement (CR 614.11, 616.1, 121.2), "as this enters"
//! replacements of permanents entering together (CR 614.12, Thief of Blood), effects that
//! add to the counters a permanent is put onto the battlefield with or gets (Prairie Dog,
//! Combine Guildmage, High Score; CR 122.6, 616.1), Spark Rupture's planeswalkers entering
//! as creatures (CR 306.5b, 613), and finality counters (CR 122.1h).

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
    let s = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::Searched { player: P0, .. }))
        .count();
    let sh = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player: P0 }))
        .count();
    (s, sh)
}

#[test]
fn archmage_ascension_ignores_cards_put_into_hand_without_drawing() {
    cr!("121.2", "614.11");
    ruling!(
        "Archmage Ascension",
        "If a spell or ability causes you to put cards in your hand without specifically using the word “draw,” Archmage Ascension’s abilities ignore it."
    );
    supported("Archmage Ascension");
    supported("Raise Dead");
    // With six quest counters, returning a card to hand isn't replaced.
    let mut t = TestGame::new(2);
    let a = ascension(&mut t, 6);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Raise Dead", &[obj(bears)]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(searches(&t).0, 0);
    // Two cards put into hand aren't two cards drawn: no quest counter at the end step.
    cast_resolve(&mut t, P0, "Raise Dead", &[obj(giant)]);
    assert!(t.in_hand(P0, "Hill Giant"));
    t.answer_yes(P0, true);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(a, "quest"), 6);
}

#[test]
fn archmage_ascension_replaces_each_draw_of_several_one_at_a_time() {
    cr!("121.2", "614.11", "616.1");
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
    let drew = t
        .g
        .turn_events
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
    cr!("616.1", "616.1e", "614.5");
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
    cr!("614.12", "614.1c", "616.1");
    ruling!(
        "Thief of Blood",
        "If more than one Thief of Blood enters the battlefield at the same time, you can apply the replacement effect of any of them."
    );
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
fn combine_guildmage_gives_a_counter_to_a_creature_entering_without_any() {
    cr!("614.1c", "122.6");
    ruling!(
        "Combine Guildmage",
        "If a creature would normally enter the battlefield with no +1/+1 counters on it, Combine Guildmage’s first ability causes it to enter the battlefield with a +1/+1 counter on it."
    );
    supported("Combine Guildmage");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Combine Guildmage");
    t.lands(P0, "Forest", 2);
    t.activate(P0, gm, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

/// P0's Combine Guildmage moves a +1/+1 counter from `from` onto `to`; `meddle` runs with
/// the ability on the stack.
fn guildmage_move(
    t: &mut TestGame,
    from: ObjectId,
    to: ObjectId,
    meddle: impl FnOnce(&mut TestGame),
) {
    let gm = t.battlefield(P0, "Combine Guildmage");
    t.lands(P0, "Island", 2);
    t.activate(P0, gm, 1, &[obj(from), obj(to)]).unwrap();
    meddle(t);
    t.resolve_all();
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
fn combine_guildmage_moves_nothing_from_a_creature_without_counters() {
    cr!("122.5", "609.3");
    ruling!(
        "Combine Guildmage",
        "If the first target creature for Combine Guildmage’s second ability has no +1/+1 counters on it as the ability resolves, nothing happens."
    );
    let mut t = TestGame::new(2);
    let a = with_counters(&mut t, P0, "Grizzly Bears", 1);
    let b = t.battlefield(P0, "Hill Giant");
    guildmage_move(&mut t, a, b, |t| remove_plus1(t, a, 1));
    assert_eq!((plus1(&t, a), plus1(&t, b)), (0, 0));
}

#[test]
fn two_high_scores_each_add_one_counter() {
    cr!("616.1", "614.1a");
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

// --- Spark Rupture, Emperor of Bones -----------------------------------------------------

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

#[test]
fn emperor_of_bones_finality_counter_exiles_instead_of_the_graveyard() {
    cr!("122.1h", "614.6");
    ruling!(
        "Emperor of Bones",
        "If any permanent with a finality counter on it would go to a graveyard from the battlefield, exile it instead."
    );
    supported("Emperor of Bones");
    let mut t = TestGame::new(2);
    let emperor = t.battlefield(P0, "Emperor of Bones");
    let bears = t.graveyard(P1, "Grizzly Bears");
    // The Emperor exiles P1's Bears at the beginning of combat, then reanimates it with a
    // finality counter when it adapts.
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(t.g.current(bears))]);
    t.activate(P0, emperor, 0, &[]).unwrap();
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "finality"), 1);
    destroy(&mut t, back[0]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
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
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1, "only the real Siege's");
    t.resolve_all();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
}
