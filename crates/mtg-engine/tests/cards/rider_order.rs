//! Order of the instructions replacement effects leave for after their modified events:
//! regeneration riders ("When it regenerates this way, ...") and "... instead. When you
//! do, ..." riders, when several are left at once by simultaneous events. They're
//! triggered abilities (CR 603.12) put on the stack in the order the effects were applied
//! unless their controller orders them otherwise (CR 603.3b).

use crate::basic_effects_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The items of the (only) trigger-ordering decision P0 was asked, bottom of stack first.
fn trigger_order(t: &TestGame, p: PlayerId) -> Vec<String> {
    let orders: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::Order { prompt, items } if q == p && prompt.contains("triggered") => {
                Some(items)
            }
            _ => None,
        })
        .collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    orders.into_iter().next().unwrap()
}

/// Two creatures with regeneration shields with riders, destroyed at once (Day of
/// Judgment); `first` is put onto the battlefield (and so destroyed and regenerated) first.
fn regenerate_both(golem_first: bool) -> Vec<String> {
    let mut t = TestGame::new(2);
    let (golem, skel) = if golem_first {
        let g = t.battlefield(P0, "Matopi Golem");
        (g, t.enter(P0, "Skeleton Scavengers"))
    } else {
        let s = t.enter(P0, "Skeleton Scavengers");
        (t.battlefield(P0, "Matopi Golem"), s)
    };
    t.lands(P0, "Plains", 6);
    t.activate(P0, golem, 0, &[]).unwrap();
    t.resolve();
    t.activate(P0, skel, 0, &[]).unwrap();
    t.resolve();
    let doj = t.hand(P0, "Day of Judgment");
    t.cast(P0, doj).go();
    t.resolve();
    assert!(t.on_battlefield(golem) && t.on_battlefield(skel));
    assert_eq!(t.stack_len(), 2);
    // The default order keeps the listed order: the first listed resolves last.
    t.resolve_all();
    assert_eq!(t.counters(golem, "-1/-1"), 1);
    assert_eq!(t.counters(skel, "+1/+1"), 2);
    trigger_order(&t, P0)
}

#[test]
fn regeneration_riders_are_listed_in_the_order_the_shields_were_used() {
    cr!("701.19a", "603.12", "603.3b");
    interp!("rider-order");
    // The permanents are destroyed (and the shields used) in the order they're on the
    // battlefield.
    assert_eq!(
        regenerate_both(true),
        vec![
            "Matopi Golem: reflexive trigger".to_string(),
            "Skeleton Scavengers: reflexive trigger".to_string(),
        ]
    );
    assert_eq!(
        regenerate_both(false),
        vec![
            "Skeleton Scavengers: reflexive trigger".to_string(),
            "Matopi Golem: reflexive trigger".to_string(),
        ]
    );
}

#[test]
fn when_you_do_riders_are_listed_in_the_order_the_effects_were_applied() {
    cr!("614.6", "616.1", "603.12", "603.3b");
    interp!("rider-order");
    // Two of P1's creatures die at once; Head of the Hunt and Nemata both apply to each, and
    // P1 picks a different one for each creature, in both possible ways.
    for (a, b) in [(0, 1), (1, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Head of the Hunt");
        t.battlefield(P0, "Nemata, Primeval Warden");
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        t.answer(P1, DecisionKind::Replacement, Answer::Index(a));
        t.answer(P1, DecisionKind::Replacement, Answer::Index(b));
        t.lands(P0, "Plains", 4);
        let doj = t.hand(P0, "Day of Judgment");
        t.cast(P0, doj).go();
        t.resolve();
        assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
        // The effect P1 applied first, by its source's name.
        let chosen: Vec<String> = t
            .asked()
            .into_iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if p == P1 => Some(options),
                _ => None,
            })
            .zip([a, b])
            .map(|(options, i)| options[i].clone())
            .collect();
        assert_eq!(chosen.len(), 2);
        let order = trigger_order(&t, P0);
        assert_eq!(order.len(), 2);
        for (o, c) in order.iter().zip(&chosen) {
            let source = o.split(':').next().unwrap();
            assert!(c.contains(source), "{order:?} vs {chosen:?}");
        }
    }
}
