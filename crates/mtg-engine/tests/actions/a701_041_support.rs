//! CR 701.41: support.

use crate::a701_028_071_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn target_candidates(t: &TestGame) -> Vec<(Vec<Entity>, u32, u32)> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                candidates,
                min,
                max,
                ..
            } => Some((candidates, min, max)),
            _ => None,
        })
        .collect()
}

#[test]
fn support_on_a_spell_puts_a_counter_on_each_of_up_to_n_target_creatures() {
    cr!("701.41a");
    ruling!("Gladehart Cavalry", "Support can target a creature you don’t control.");
    supported("Shoulder to Shoulder");
    // "Support 2. Draw a card."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Shoulder to Shoulder");
    let hand = t.hand_size(P0);
    t.cast(P0, spell)
        .targets(&[Entity::Object(mine), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand); // cast one, drew one
    let asked = target_candidates(&t);
    assert_eq!((asked[0].1, asked[0].2), (0, 2));
}

#[test]
fn support_on_a_permanent_targets_other_creatures() {
    cr!("701.41a");
    ruling!(
        "Gladehart Cavalry",
        "You can’t put more than one +1/+1 counter on any one target using the support action."
    );
    supported("Gladehart Cavalry");
    // "Support 6." — "When this creature enters, put a +1/+1 counter on each of up to six
    // other target creatures."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    for _ in 0..5 {
        t.battlefield(P1, "Llanowar Elves");
    }
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    let cavalry = t.enter(P0, "Gladehart Cavalry");
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.counters(cavalry, "+1/+1"), 0);
    let asked = target_candidates(&t);
    assert_eq!(asked.len(), 1);
    let (cands, min, max) = &asked[0];
    assert!(!cands.contains(&Entity::Object(cavalry)));
    assert!(cands.contains(&Entity::Object(a)) && cands.contains(&Entity::Object(b)));
    assert_eq!(cands.len(), 7);
    assert_eq!((*min, *max), (0, 6));
}
