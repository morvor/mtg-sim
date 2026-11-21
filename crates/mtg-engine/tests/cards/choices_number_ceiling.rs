//! "Choose a number" with no upper limit in the text (Menacing Ogre, Wheel of Misfortune,
//! Goblin Game, Void, Mindblaze): the decision's range goes up to
//! `choices::MAX_CHOSEN_NUMBER`, and an answer outside it falls back to the least number.

use mtg_engine::choices::MAX_CHOSEN_NUMBER;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn number(t: &mut TestGame, p: PlayerId, n: i64) {
    t.answer(p, DecisionKind::Number, Answer::Number(n));
}

/// The (min, max) of every number decision asked.
fn ranges(t: &TestGame) -> Vec<(i64, i64)> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseNumber { min, max, .. } => Some((min, max)),
            _ => None,
        })
        .collect()
}

#[test]
fn menacing_ogre_numbers_far_above_a_thousand_can_be_chosen() {
    cr!("608.2c", "107.1");
    interp!("unbounded-number-ceiling");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    number(&mut t, P0, 2_000);
    number(&mut t, P1, 1_500);
    t.enter(P0, "Menacing Ogre");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20 - 2_000, 20));
    assert_eq!(
        ranges(&t),
        vec![(0, MAX_CHOSEN_NUMBER as i64), (0, MAX_CHOSEN_NUMBER as i64)]
    );
}

#[test]
fn menacing_ogre_the_ceiling_itself_and_an_answer_above_it() {
    cr!("608.2c");
    interp!("unbounded-number-ceiling", "default-numbers");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    number(&mut t, P0, MAX_CHOSEN_NUMBER as i64);
    // Above the ceiling: invalid, so P1 chooses the least number (0).
    number(&mut t, P1, MAX_CHOSEN_NUMBER as i64 + 1);
    t.enter(P0, "Menacing Ogre");
    t.resolve_all();
    assert_eq!(t.life(P0), 20 - MAX_CHOSEN_NUMBER);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn wheel_of_misfortune_and_goblin_game_take_large_numbers() {
    cr!("608.2c", "107.1b");
    interp!("unbounded-number-ceiling");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wheel = t.hand(P0, "Wheel of Misfortune");
    t.advance_to(P0, Step::PrecombatMain);
    number(&mut t, P0, 0);
    number(&mut t, P1, 5_000);
    t.cast(P0, wheel).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20 - 5_000));
    assert!(ranges(&t)
        .iter()
        .all(|r| *r == (0, MAX_CHOSEN_NUMBER as i64)));

    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let game = t.hand(P0, "Goblin Game");
    t.advance_to(P0, Step::PrecombatMain);
    number(&mut t, P0, 1_234);
    number(&mut t, P1, 1_235);
    t.cast(P0, game).go();
    t.resolve_all();
    // P0 revealed the fewest; half of a negative life total is nothing to lose.
    assert_eq!((t.life(P0), t.life(P1)), (20 - 1_234, 20 - 1_235));
    assert!(ranges(&t)
        .iter()
        .all(|r| *r == (1, MAX_CHOSEN_NUMBER as i64)));
}

#[test]
fn void_and_mindblaze_ask_for_a_number_without_a_small_ceiling() {
    cr!("107.1");
    interp!("unbounded-number-ceiling");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let void = t.hand(P0, "Void");
    t.advance_to(P0, Step::PrecombatMain);
    number(&mut t, P0, 1_001);
    t.cast(P0, void).target(P1).go();
    t.resolve_all();
    assert_eq!(ranges(&t), vec![(0, MAX_CHOSEN_NUMBER as i64)]);
    // 1001 was chosen, not the default 0: the Ornithopter (mana value 0) survives.
    assert!(t.on_battlefield(thopter));

    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    let mb = t.hand(P0, "Mindblaze");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, mb).target(P1).go();
    t.resolve_all();
    assert!(ranges(&t).contains(&(1, MAX_CHOSEN_NUMBER as i64)));
}
