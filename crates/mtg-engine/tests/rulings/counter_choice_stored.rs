//! A choice of what to put counters on offers only the objects that can have them (register
//! `counter-choice-excludes-what-cant-have-them`, decided by the user, Nov 20: "Accept and
//! fix"), also for a choice the compiled text makes first and stores for the instruction
//! that puts the counters (Eye of Doom's "each player chooses a nonland permanent and puts a
//! doom counter on it", Promise of Loyalty's "Each player puts a vow counter on a creature
//! they control and sacrifices the rest") and for bolster's own choice (CR 701.39a), with
//! Melira's Keepers ("This creature can't have counters put on it.") among the options.

use crate::r_residue_counters_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` was offered `what` in a choice of objects since decision `from`, and whether
/// they were asked to choose among objects at all.
fn offered(t: &TestGame, p: PlayerId, from: usize, what: ObjectId) -> (bool, bool) {
    let choices: Vec<Vec<Entity>> = asked_since(t, from)
        .into_iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    (
        choices.iter().any(|c| c.contains(&obj(what))),
        !choices.is_empty(),
    )
}

#[test]
fn eye_of_doom_cant_choose_a_permanent_that_cant_have_counters() {
    cr!("608.2d", "122.1");
    interp!("counter-choice-excludes-what-cant-have-them");
    ruling!(
        "Eye of Doom",
        "The active player chooses a permanent first, followed by each other player in turn order."
    );
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    supported("Eye of Doom");
    supported("Melira's Keepers");
    // Eye of Doom enters under P0's control: each player chooses a nonland permanent and
    // puts a doom counter on it. Melira's Keepers (P1's) isn't offered to either player; P1
    // tries to choose it anyway and gets the default, another permanent.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let keepers = t.battlefield(P1, "Melira's Keepers");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(keepers)]);
    let from = t.asked().len();
    let eye = enter_together(&mut t, &[(P0, "Eye of Doom")])[0];
    t.settle();
    t.resolve_all();
    for p in [P0, P1] {
        let (keepers_offered, asked) = offered(&t, p, from, keepers);
        assert!(asked, "{p:?} chose a permanent\n{}", t.dump_log());
        assert!(!keepers_offered, "{p:?} was offered Melira's Keepers");
    }
    // The active player chose first, then the other player.
    let choosers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(choosers, vec![P0, P1], "{}", t.dump_log());
    assert_eq!(t.counters(keepers, "doom"), 0);
    assert!(t.counters(bears, "doom") >= 1, "P0's choice");
    // P1's invalid answer: another nonland permanent got P1's doom counter (by default the
    // first one offered, the Bears again or Eye of Doom).
    assert_eq!(
        t.counters(bears, "doom") + t.counters(eye, "doom"),
        2,
        "{}",
        t.dump_log()
    );
}

#[test]
fn promise_of_loyalty_vow_counter_goes_on_a_creature_that_can_have_it() {
    cr!("608.2d", "122.1", "701.21a");
    interp!("counter-choice-excludes-what-cant-have-them");
    ruling!(
        "Promise of Loyalty",
        "Each player, including you, must put a vow counter on a creature they control if able. They can't choose to opt out and sacrifice all of their creatures."
    );
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    supported("Promise of Loyalty");
    supported("Melira's Keepers");
    // P1 controls Melira's Keepers and Grizzly Bears and tries to keep the Keepers: only the
    // Bears can have the vow counter, so P1 must put it on the Bears and sacrifices the
    // Keepers.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let keepers = t.battlefield(P1, "Melira's Keepers");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.answer_choose(P1, &[obj(keepers)]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Promise of Loyalty", &[]);
    t.resolve_all();
    let (keepers_offered, _) = offered(&t, P1, from, keepers);
    assert!(!keepers_offered, "{}", t.dump_log());
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "vow"), 1);
    assert!(!t.on_battlefield(keepers), "the rest are sacrificed");
    assert!(t.on_battlefield(wall));
    assert_eq!(t.counters(wall, "vow"), 1);
}

#[test]
fn promise_of_loyalty_with_only_a_creature_that_cant_have_counters_sacrifices_it() {
    cr!("608.2d", "609.3", "701.21a");
    interp!("counter-choice-excludes-what-cant-have-them");
    interp!("kept-for-counters-none-able-all-sacrificed");
    ruling!(
        "Promise of Loyalty",
        "Each player, including you, must put a vow counter on a creature they control if able. They can't choose to opt out and sacrifice all of their creatures."
    );
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    supported("Promise of Loyalty");
    supported("Melira's Keepers");
    // Melira's Keepers is P1's only creature: no vow counter can be put on it, so P1 puts
    // none, keeps no creature "this way" and sacrifices the rest, the Keepers.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let keepers = t.battlefield(P1, "Melira's Keepers");
    t.answer_choose(P1, &[obj(keepers)]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Promise of Loyalty", &[]);
    t.resolve_all();
    let (keepers_offered, _) = offered(&t, P1, from, keepers);
    assert!(!keepers_offered, "{}", t.dump_log());
    assert!(!t.on_battlefield(keepers), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Melira's Keepers"));
}

#[test]
fn bolster_chooses_among_the_least_toughness_creatures_that_can_have_counters() {
    cr!("701.39a", "608.2d");
    interp!("counter-choice-excludes-what-cant-have-them");
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    supported("Honor's Reward");
    supported("Melira's Keepers");
    // Melira's Keepers (4/4) and Serra Angel (4/4) are tied for the least toughness among
    // P0's creatures: only the Angel can have the counters, so it gets them even though P0
    // tries to choose the Keepers.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let keepers = t.battlefield(P0, "Melira's Keepers");
    let angel = t.battlefield(P0, "Serra Angel");
    t.answer_choose(P0, &[obj(keepers)]);
    cast_new(&mut t, P0, "Honor's Reward", &[]);
    t.resolve_all();
    assert_eq!(t.counters(angel, counters::PLUS1), 2, "{}", t.dump_log());
    assert_eq!(t.counters(keepers, counters::PLUS1), 0);
}

#[test]
fn bolster_with_only_a_creature_that_cant_have_counters_at_the_least_toughness_does_nothing() {
    cr!("701.39a", "608.2d");
    interp!("counter-choice-excludes-what-cant-have-them");
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    supported("Honor's Reward");
    supported("Melira's Keepers");
    // Melira's Keepers (4/4) alone has the least toughness among P0's creatures; Craw Wurm
    // has a +1/+1 counter (7/5). Bolster's creature is one with the least toughness among
    // all of P0's creatures (CR 701.39a), the Keepers, which can't have the counters: no
    // creature gets them, not the Wurm either. The rest of the spell still happens.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let keepers = t.battlefield(P0, "Melira's Keepers");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.g.add_counters(Entity::Object(wurm), counters::PLUS1, 1, None);
    let life = t.life(P0);
    cast_new(&mut t, P0, "Honor's Reward", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), life + 4, "{}", t.dump_log());
    assert_eq!(t.counters(keepers, counters::PLUS1), 0);
    assert_eq!(t.counters(wurm, counters::PLUS1), 1, "{}", t.dump_log());
}
