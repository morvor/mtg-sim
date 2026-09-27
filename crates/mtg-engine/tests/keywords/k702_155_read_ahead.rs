//! CR 702.155 Read ahead.

use crate::common_k702_153_167::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// The Elder Dragon War (read ahead): "I — This Saga deals 2 damage to each creature and
// each opponent. II — Discard any number of cards, then draw that many cards. III —
// Create a 4/4 red Dragon creature token with flying."
const WAR: &str = "The Elder Dragon War";

fn lore(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::LORE)
}

fn dragons(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Dragon"))
        .count()
}

/// Puts `n` lore counters on the Saga, as an effect would.
fn add_lore(t: &mut TestGame, saga: ObjectId, n: i32) {
    run_effect(
        t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: counters::LORE.into(),
            n: Value::c(n),
        },
        &[Entity::Object(saga)],
    );
}

/// The read ahead questions `p` was asked: (min, max).
fn read_ahead_questions(t: &TestGame) -> Vec<(i64, i64)> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseNumber { min, max, prompt, .. } if prompt.contains("Read ahead") => {
                Some((min, max))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_read_ahead_saga_enters_with_the_chosen_number_of_lore_counters() {
    cr!("702.155", "702.155b");
    ruling!(
        "The Cruelty of Gix",
        "Neither choosing the number nor putting the counters on the Saga use the stack, and neither can be responded to."
    );
    ruling!(
        "The Elder Dragon War",
        "As a Saga with read ahead enters the battlefield, its controller chooses a number from one to that Saga's greatest chapter number."
    );
    assert_supported(WAR);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, WAR);
    t.cast(P0, card).go();
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.resolve();
    let saga = named(&t, P0, WAR)[0];
    // A number between one and its final chapter number, chosen as it enters.
    assert_eq!(read_ahead_questions(&t), vec![(1, 3)]);
    assert_eq!(lore(&t, saga), 3);
    // Only chapter III triggers: skipped chapters don't.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(dragons(&t), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn chapters_trigger_the_turn_it_entered_only_for_exactly_that_many_counters() {
    cr!("702.155a");
    // Enters with one counter; two more the same turn skip chapter II: only chapter III
    // (exactly three counters) triggers.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    let saga = t.enter(P0, WAR);
    t.settle();
    assert_eq!(t.stack_len(), 1, "chapter I");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    add_lore(&mut t, saga, 2);
    t.settle();
    assert_eq!(lore(&t, saga), 3);
    assert_eq!(t.stack_len(), 1, "chapter III only");
    t.resolve_all();
    assert_eq!(dragons(&t), 1);
    // Enters with two: chapter II triggers (exactly two), not chapter I; one more the
    // same turn is exactly three: chapter III triggers.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let saga = t.enter(P0, WAR);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "chapter I didn't trigger");
    add_lore(&mut t, saga, 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(dragons(&t), 1);
}

#[test]
fn after_the_turn_it_entered_chapters_trigger_normally() {
    cr!("702.155a");
    // A read ahead Saga that entered on an earlier turn: two counters at once trigger both
    // chapters they pass.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    let saga = t.enter(P0, WAR);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    add_lore(&mut t, saga, 2);
    t.settle();
    assert_eq!(t.stack_len(), 2, "chapters II and III");
}

#[test]
fn several_instances_of_read_ahead_are_redundant() {
    cr!("702.155c");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let def = custom_card(
        "Twice Told Tale",
        "Enchantment — Saga",
        None,
        "Read ahead, read ahead\nI — You gain 1 life.\nII — You gain 2 life.\nIII — You gain 3 life.",
    );
    let saga = enter_def(&mut t, P0, def);
    assert_eq!(kw_count(&t, saga, KeywordKind::ReadAhead), 2);
    // One choice, one set of counters.
    assert_eq!(read_ahead_questions(&t), vec![(1, 3)]);
    assert_eq!(lore(&t, saga), 2);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn barbara_wright_gives_sagas_read_ahead_as_they_enter() {
    cr!("702.155b", "702.155c", "614.12");
    ruling!(
        "Barbara Wright",
        "Having multiple instances of read ahead doesn't cause anything unusual to happen."
    );
    assert_supported("Barbara Wright");
    // Barbara Wright: "Sagas you control have read ahead." History of Benalia (I, II:
    // create a 2/2 Knight with vigilance) has no read ahead of its own: it gets it as it
    // enters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barbara Wright");
    t.lands(P0, "Plains", 3);
    let card = t.hand(P0, "History of Benalia");
    t.cast(P0, card).go();
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.resolve();
    let saga = named(&t, P0, "History of Benalia")[0];
    assert_eq!(read_ahead_questions(&t), vec![(1, 3)]);
    assert_eq!(lore(&t, saga), 2);
    // Only chapter II triggers.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);

    // The Elder Dragon War has read ahead and gets a second instance: still one choice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barbara Wright");
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, WAR);
    t.cast(P0, card).go();
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.resolve();
    let saga = named(&t, P0, WAR)[0];
    assert_eq!(kw_count(&t, saga, KeywordKind::ReadAhead), 2);
    assert_eq!(read_ahead_questions(&t), vec![(1, 3)]);
    assert_eq!(lore(&t, saga), 3);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(dragons(&t), 1);
}
