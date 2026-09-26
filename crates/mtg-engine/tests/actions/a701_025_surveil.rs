//! CR 701.25: surveil.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn surveil(t: &mut TestGame, p: PlayerId, n: i32) {
    run(
        t,
        p,
        None,
        Effect::Surveil {
            who: PlayerRef::You,
            n: Value::c(n),
        },
    );
    t.resolve_all();
}

fn surveilled_cards(t: &TestGame) -> Vec<Vec<ObjectId>> {
    t.asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Surveil { cards } => Some(cards.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn additional_cards_looked_at_are_among_the_cards_surveilled() {
    cr!("701.25", "701.25b");
    ruling!(
        "Enhanced Surveillance",
        "The additional cards you look at due to Enhanced Surveillance's ability are part of what you surveil"
    );
    supported("Enhanced Surveillance");
    let mut t = TestGame::new(2);
    // "You may look at an additional two cards each time you surveil."
    t.battlefield(P0, "Enhanced Surveillance");
    let c = t.library_top(P0, "Forest");
    let b = t.library_top(P0, "Island");
    let a = t.library_top(P0, "Swamp");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![c, a], vec![b]));
    surveil(&mut t, P0, 1);
    assert_eq!(surveilled_cards(&t), vec![vec![a, b, c]]);
    assert!(t.in_graveyard(P0, "Island"));
    let lib = &t.g.player(P0).library;
    assert_eq!(&lib[lib.len() - 2..], &[a, c]);
}

#[test]
fn additional_cards_from_two_sources_add_up() {
    cr!("701.25b");
    ruling!(
        "Enhanced Surveillance",
        "their effects both apply and you may look at an additional four cards"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enhanced Surveillance");
    t.battlefield(P0, "Enhanced Surveillance");
    // An opponent's doesn't apply to P0.
    t.battlefield(P1, "Enhanced Surveillance");
    t.answer_yes(P0, true);
    surveil(&mut t, P0, 2);
    assert_eq!(surveilled_cards(&t)[0].len(), 6);
    // Declining to look at more.
    t.answer_yes(P0, false);
    surveil(&mut t, P0, 2);
    assert_eq!(surveilled_cards(&t)[1].len(), 2);
}

#[test]
fn surveilling_zero_is_no_surveil_event() {
    cr!("701.25c");
    supported("Dimir Spybug");
    let mut t = TestGame::new(2);
    // "Whenever you surveil, put a +1/+1 counter on this creature."
    let bug = t.battlefield(P0, "Dimir Spybug");
    surveil(&mut t, P0, 0);
    assert_eq!(t.counters(bug, "+1/+1"), 0);
    assert!(surveilled_cards(&t).is_empty());
    surveil(&mut t, P0, 1);
    assert_eq!(t.counters(bug, "+1/+1"), 1);
}

#[test]
fn surveilling_with_an_empty_library_still_triggers() {
    cr!("701.25d");
    ruling!("Dimir Spybug", "It even triggers if you have no cards in your library.");
    let mut t = TestGame::new(2);
    let bug = t.battlefield(P0, "Dimir Spybug");
    clear_library(&mut t, P0);
    surveil(&mut t, P0, 2);
    assert_eq!(t.counters(bug, "+1/+1"), 1);
    assert_eq!(t.zone(bug), Zone::Battlefield);
}
