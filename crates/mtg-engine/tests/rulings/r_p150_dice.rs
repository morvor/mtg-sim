//! Rulings batch P150 — die rolls and Attractions: Swinging Ship's additional combats,
//! Clowning Around's Robot count, Atomwheel Acrobats' roll, Centaur of Attention's stored
//! results.

use crate::r_s01_common::*;
use crate::r_s21_common::go_to;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s35_common::{die_rolls, load_dice};
use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` rolls to visit their Attractions with the given die result.
fn roll_to_visit(t: &mut TestGame, p: PlayerId, result: u32) {
    t.g.dice.loaded.push_back(result);
    mtg_engine::variants::roll_to_visit(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

fn lit(t: &TestGame, attraction: ObjectId) -> u32 {
    t.obj_now(attraction)
        .card
        .as_ref()
        .unwrap()
        .attraction_lights[0] as u32
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Runs the game to the next beginning of combat step of this turn (the step's triggers
/// resolved), declaring `attack` in the declare attackers step of the combat it leaves.
fn next_combat(t: &mut TestGame) {
    let turn = t.g.turn.number;
    // Leave the current combat phase (if any) first.
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::EndOfCombat || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn);
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::BeginningOfCombat
            && g.turn.stage == mtg_engine::turn::Stage::Priority)
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "no further combat this turn");
    t.resolve_all();
}

fn combats_this_turn(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                mtg_engine::events::Event::StepBegan {
                    step: Step::BeginningOfCombat,
                    ..
                }
            )
        })
        .count()
}

#[test]
fn swinging_ship_visited_twice_gives_two_combats_that_each_untap_attackers() {
    cr!("500.8", "506.1", "603.7a", "701.52a");
    ruling!(
        "Swinging Ship",
        "If you visit Swinging Ship multiple times before the first combat phase of a turn has ended, that many additional combat phases will happen, one at a time. At the beginning of each of those combats, any creature that attacked during any combat that turn will untap."
    );
    supported("Swinging Ship");
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Swinging Ship");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let n = lit(&t, ship);
    roll_to_visit(&mut t, P0, n);
    t.resolve_all();
    roll_to_visit(&mut t, P0, n);
    t.resolve_all();
    // Combat 1: the Bears attack.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    go_to(&mut t, Step::BeginningOfCombat);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(tapped(&t, bears));
    // Combat 2: untapped at its beginning; they attack again.
    next_combat(&mut t);
    assert!(!tapped(&t, bears));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(tapped(&t, bears));
    // Combat 3: untapped again.
    next_combat(&mut t);
    assert!(!tapped(&t, bears));
    assert_eq!(combats_this_turn(&t), 3);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(t.life(P1), 14);
    assert_eq!(combats_this_turn(&t), 3);
}

#[test]
fn swinging_ship_doesnt_untap_creatures_that_entered_attacking() {
    cr!("508.4", "701.52a");
    ruling!(
        "Swinging Ship",
        "Swinging Ship won’t untap any creatures that entered the battlefield attacking. Those creatures never attacked, meaning they were never declared as attacking creatures."
    );
    supported("Hero of Bladehold");
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Swinging Ship");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    t.set_step(P0, Step::PrecombatMain);
    let n = lit(&t, ship);
    roll_to_visit(&mut t, P0, n);
    t.resolve_all();
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(hero, Entity::Player(P1))]),
    );
    go_to(&mut t, Step::BeginningOfCombat);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    let soldiers = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    assert!(soldiers.iter().all(|s| tapped(&t, *s)));
    next_combat(&mut t);
    assert!(!tapped(&t, hero));
    assert!(soldiers.iter().all(|s| tapped(&t, *s)));
}

#[test]
fn clowning_around_counts_only_creatures_with_the_robot_type() {
    cr!("205.3m", "706.2");
    ruling!(
        "Clowning Around",
        "Robot is a creature type. There may be creatures that appear to be robotic in nature that don't have the Robot creature type. They don't count."
    );
    supported("Clowning Around");
    // Ornithopter (an artifact Thopter) isn't a Robot: two Robots, a 3 is too high.
    for (roll, made) in [(3, 2), (2, 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ornithopter");
        load_dice(&mut t, &[roll]);
        cast_and_resolve(&mut t, P0, "Clowning Around", &[]);
        assert_eq!(die_rolls(&t).len(), 1);
        assert_eq!(tokens(&t, P0).len(), made, "rolled {roll}");
    }
}

#[test]
fn atomwheel_acrobats_roll_triggers_roll_abilities() {
    cr!("706.2", "603.2");
    ruling!(
        "Atomwheel Acrobats",
        "The die roll caused by the activated ability has no inherent effect: you just roll a six-sided die, but any ability that triggers whenever you roll a die will trigger."
    );
    supported("Atomwheel Acrobats");
    supported("Brazen Dwarf");
    for (roll, counters) in [(5, 0), (2, 2)] {
        let mut t = TestGame::new(2);
        let acro = t.battlefield(P0, "Atomwheel Acrobats");
        t.battlefield(P0, "Brazen Dwarf");
        t.lands(P0, "Forest", 3);
        load_dice(&mut t, &[roll]);
        t.activate(P0, acro, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P1), 19, "Brazen Dwarf triggered");
        assert_eq!(t.counters(acro, "+1/+1"), counters);
    }
}

/// The stored results on the object, sorted.
fn stored(t: &TestGame, id: ObjectId) -> Vec<i64> {
    let mut v: Vec<i64> =
        t.g.dice
            .stored
            .get(&id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(sides, r)| {
                assert_eq!(sides, 6);
                r
            })
            .collect();
    v.sort();
    v
}

#[test]
fn centaur_of_attention_rerolls_replace_stored_results_even_ones_kept_before() {
    cr!("706.8", "706.8b");
    ruling!(
        "Centaur of Attention",
        "To store a result means to note the result, ya see. You can do it by keeping a die with that result on Centaur of Attention, or just by writing down the result. To reroll a noted result, roll a die with the same number of sides and replace the noted result with the new result."
    );
    ruling!(
        "Centaur of Attention",
        "You may reroll any results stored on Centaur of Attention, even those you chose not to reroll on previous turns."
    );
    let mut t = TestGame::new(2);
    load_dice(&mut t, &[1, 2, 3, 4, 5]);
    let centaur = t.enter(P0, "Centaur of Attention");
    t.resolve_all();
    assert_eq!(stored(&t, centaur), vec![1, 2, 3, 4, 5]);
    // Turn 1: reroll only the first stored result (the 1), into a 5.
    t.answer_yes(P0, true);
    for y in [true, false, false, false, false] {
        t.answer_yes(P0, y);
    }
    load_dice(&mut t, &[5]);
    t.set_step(P0, Step::PrecombatMain);
    go_to(&mut t, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(stored(&t, centaur), vec![2, 3, 4, 5, 5]);
    // A later turn: reroll the 2, 3 and 4, kept last time; each new result replaces one.
    t.answer_yes(P0, true);
    load_dice(&mut t, &[5, 5, 6]);
    let order: Vec<i64> = t.g.dice.stored[&centaur].iter().map(|r| r.1).collect();
    for r in order {
        t.answer_yes(P0, matches!(r, 2..=4));
    }
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(stored(&t, centaur), vec![5, 5, 5, 5, 6]);
    assert!(die_rolls(&t).iter().all(|r| r.1 == 6));
    t.g.recompute();
    assert_eq!(
        t.pt(centaur).0,
        card("Centaur of Attention").front().chars.power.unwrap() + 4
    );
}
