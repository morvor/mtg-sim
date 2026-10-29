//! Rulings batch S33 — "if you gained life this turn" at the beginning of the end step
//! (CR 603.4: the condition is checked as the end step begins; life gained later doesn't
//! make the ability trigger), and "the amount of life you gained this turn" (the life
//! gained, not counting life lost, CR 119.3).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s04_common::run_with;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 gains (or, negative, loses) `n` life from an effect P0 controls.
fn life_change(t: &mut TestGame, n: i32) {
    let effect = if n >= 0 {
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::c(n),
        }
    } else {
        Effect::LoseLife {
            who: PlayerRef::You,
            n: Value::c(-n),
        }
    };
    run_with(t, P0, effect, &[]);
}

/// The Demon tokens P0 controls (their P/T).
fn demons(t: &TestGame) -> Vec<(i32, i32)> {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Demon"))
        .map(|o| (o.power(), o.toughness()))
        .collect()
}

#[test]
fn tivash_you_need_to_gain_life_before_the_end_step_begins() {
    cr!("603.4", "513.1");
    ruling!(
        "Tivash, Gloom Summoner",
        "You need to gain life before the end step begins for the last ability to trigger."
    );
    supported("Tivash, Gloom Summoner");
    // "At the beginning of your end step, if you gained life this turn, you may pay X
    // life, where X is the amount of life you gained this turn. If you do, create an X/X
    // black Demon creature token with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tivash, Gloom Summoner");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Demon"), 0);
    // Gaining life during the end step is too late.
    life_change(&mut t, 3);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Demon"), 0);
    t.resolve_all();
    assert!(demons(&t).is_empty());
}

#[test]
fn tivash_counts_life_gained_without_life_lost_and_pays_exactly_that() {
    cr!("603.4", "107.3c", "119.4");
    ruling!(
        "Tivash, Gloom Summoner",
        "Tivash’s ability counts the total amount of life gained without considering any life you lost during that turn. For example, if you lost 3 life and gained 4 life earlier in the turn, you may pay 4 life to create a 4/4 Demon."
    );
    ruling!(
        "Tivash, Gloom Summoner",
        "You can’t pay less life than the amount of life you gained to create a smaller but less harmful Demon."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tivash, Gloom Summoner");
    life_change(&mut t, -3);
    life_change(&mut t, 4);
    assert_eq!(t.life(P0), 21);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Demon"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    // P0 paid 4 life (the amount gained) for a 4/4 flying Demon.
    assert_eq!(t.life(P0), 17);
    assert_eq!(demons(&t), vec![(4, 4)]);
    // Declining: no Demon, no life paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tivash, Gloom Summoner");
    life_change(&mut t, 2);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Demon"), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(demons(&t).is_empty());
}
