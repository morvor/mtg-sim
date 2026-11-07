//! Vedalken Squirrel-Whacker (hand-written, `src/cards/vedalken_squirrel_whacker.rs`):
//! "If you would roll one or more six-sided dice, instead roll them and you may exchange
//! one result with this creature's base power or base toughness." Die results are loaded
//! (`DiceState::loaded`).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

const WHACKER: &str = "Vedalken Squirrel-Whacker";

/// The results of this turn's die rolls.
fn results(t: &TestGame) -> Vec<u32> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::DieRolled {
                result,
                planar: false,
                ..
            } => Some(*result),
            _ => None,
        })
        .collect()
}

/// The exchange prompts asked of P0, with their options.
fn exchange_prompts(t: &TestGame) -> Vec<Vec<String>> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if p == P0 && prompt.contains("base power or base toughness") => Some(options),
            _ => None,
        })
        .collect()
}

/// A 2/5 Squirrel-Whacker on P0's battlefield (its as-enters rolls showing 2 and 5).
fn whacker(t: &mut TestGame) -> ObjectId {
    t.g.dice.loaded.extend([2, 5]);
    let w = t.enter(P0, WHACKER);
    t.settle();
    assert_eq!(t.pt(w), (2, 5));
    w
}

/// P0 sacrifices Vegetation Abomination ("{T}, Sacrifice this creature: Roll a six-sided
/// die. You gain life equal to the result.") with the die showing `natural`.
fn abomination_roll(t: &mut TestGame, natural: u32) {
    let a = t.battlefield(P0, "Vegetation Abomination");
    t.g.dice.loaded.push_back(natural);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn vedalken_squirrel_whacker_compiles() {
    let def = mtg_engine::card::card(WHACKER);
    assert!(
        def.unsupported_text().is_empty(),
        "{:?}",
        def.unsupported_text()
    );
}

#[test]
fn one_result_may_be_exchanged_with_its_base_power_or_toughness() {
    cr!("706.2", "613.4b", "208.4b");
    ruling!(
        "Vedalken Squirrel-Whacker",
        "After rolling one or more six-sided dice and seeing the results, you decide whether you want to make the exchange."
    );
    // The ruling's example: a 2/5 and a roll of 3. Exchange with its base power: a 3/5,
    // and the result is 2.
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    abomination_roll(&mut t, 3);
    assert_eq!(
        exchange_prompts(&t),
        [vec![
            "Don't exchange".to_string(),
            "Exchange die 1 (result 3) with its base power 2".to_string(),
            "Exchange die 1 (result 3) with its base toughness 5".to_string(),
        ]]
    );
    assert_eq!(t.pt(w), (3, 5));
    assert_eq!(results(&t), [2, 5, 2], "after its as-enters rolls");
    assert_eq!(t.life(P0), 22, "the result is 2");

    // With its base toughness: a 2/3, and the result is 5.
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    abomination_roll(&mut t, 3);
    assert_eq!(t.pt(w), (2, 3));
    assert_eq!(t.life(P0), 25);

    // Or leave everything as it was.
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    abomination_roll(&mut t, 3);
    assert_eq!(t.pt(w), (2, 5));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn the_new_base_power_is_a_base_value_modifications_still_apply() {
    interp!("die-result-exchange-after-modifiers");
    cr!("613.4b", "613.4c");
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    t.battlefield(P0, "Glorious Anthem");
    t.settle();
    assert_eq!(t.pt(w), (3, 6));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    abomination_roll(&mut t, 6);
    // Base power 6, +1/+1 from the Anthem; the die's result is the old base power 2.
    assert_eq!(t.pt(w), (7, 6));
    assert_eq!(t.life(P0), 22);
    // It lasts (not until end of turn).
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.pt(w), (7, 6));
}

#[test]
fn only_one_result_of_several_dice() {
    cr!("706.1");
    ruling!(
        "Vedalken Squirrel-Whacker",
        "You may exchange one result at most, no matter how many dice you rolled."
    );
    // P0 rolls two six-sided dice (an effect of a P0 ability): one exchange offered.
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    let src = t.battlefield(P0, "Grizzly Bears");
    t.g.dice.loaded.extend([4, 6]);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src), P0);
    let mut spec = mtg_engine::dice::DieRoll::new(6);
    spec.count = mtg_engine::ability::Value::c(2);
    mtg_engine::dice::roll(&mut t.g, &spec, &mut ctx);
    t.g.flush_events();
    let prompts = exchange_prompts(&t);
    assert_eq!(prompts.len(), 1, "one exchange for the roll");
    assert_eq!(prompts[0].len(), 5, "either die, with power or toughness");
    // Die 2 (a 6) with its base power 2.
    assert_eq!(results(&t), [2, 5, 4, 2]);
    assert_eq!(t.pt(w), (6, 5));
}

#[test]
fn modifiers_apply_first_and_results_can_be_impossible_numbers() {
    interp!("die-result-exchange-after-modifiers");
    cr!("706.2", "706.2a");
    ruling!(
        "Vedalken Squirrel-Whacker",
        "Anything that references the \"result\" of a die roll is looking for the result after these modifications."
    );
    ruling!(
        "Vedalken Squirrel-Whacker",
        "Results can be numbers not ordinarily possible on a six-sided die."
    );
    ruling!(
        "Vedalken Squirrel-Whacker",
        "it is possible to exchange a die roll so that it has a result that would normally not be possible for that die"
    );
    // Snickering Squirrel turns a natural 6 into a 7; the exchange then sees the 7.
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    t.battlefield(P0, "Snickering Squirrel");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    abomination_roll(&mut t, 6);
    assert_eq!(t.pt(w), (2, 7), "base toughness 7");
    assert_eq!(t.life(P0), 25, "the result is the old base toughness 5");
    // Square Up gives it base power and toughness 4/4 ... until end of turn; then an
    // exchange gives a six-sided die a result of 7 (its base toughness).
    t.lands(P0, "Forest", 2);
    let sq = t.hand(P0, "Square Up");
    t.cast(P0, sq).target(Entity::Object(w)).go();
    t.resolve_all();
    assert_eq!(t.pt(w), (4, 4));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.pt(w), (2, 7));
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let life = t.life(P0);
    abomination_roll(&mut t, 1);
    assert_eq!(t.life(P0), life + 7, "a result of 7 on a six-sided die");
    assert_eq!(t.pt(w), (2, 1));
}

#[test]
fn not_its_own_as_enters_rolls_and_only_its_controllers_six_sided_dice() {
    cr!("706.1");
    ruling!(
        "Vedalken Squirrel-Whacker",
        "Vedalken Squirrel-Whacker's second ability functions only while Vedalken Squirrel-Whacker is on the battlefield."
    );
    let mut t = TestGame::new(2);
    let w = whacker(&mut t);
    assert!(
        exchange_prompts(&t).is_empty(),
        "not its own as-enters rolls"
    );
    // A d20 isn't a six-sided die.
    let src = t.battlefield(P0, "Grizzly Bears");
    t.g.dice.loaded.push_back(15);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src), P0);
    mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(20), &mut ctx);
    // An opponent's six-sided die isn't "you".
    let src1 = t.battlefield(P1, "Grizzly Bears");
    t.g.dice.loaded.push_back(6);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src1), P1);
    mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(6), &mut ctx);
    t.g.flush_events();
    assert!(exchange_prompts(&t).is_empty());
    assert_eq!(results(&t), [2, 5, 15, 6]);
    assert_eq!(t.pt(w), (2, 5));
}
