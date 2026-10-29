//! Dice (cards5-game-rules-dice, `dice`): a results table that defines X for the
//! instructions before it (Wand of Wonder), returned cards exiled again (Lae'zel's
//! Acrobatics), a roll as a cost (Clay Golem), each player's roll and "the highest result"
//! (Chaos Dragon), rolls as the permanent enters setting its base power and toughness
//! (Vedalken Squirrel-Whacker), and the next roll of a target player with an extra die
//! whose ignored roll another player chooses (Bamboozling Beeble). Die results are loaded
//! (`DiceState::loaded`).

use crate::combat2_common::supported;
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn die_rolls(t: &TestGame) -> Vec<(PlayerId, u32)> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::DieRolled {
                player,
                result,
                planar: false,
                ..
            } => Some((*player, *result)),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Wand of Wonder
// ---------------------------------------------------------------------------

/// Activates Wand of Wonder in a four-player game with the d20 showing `roll`; each
/// opponent's library has Divination on top. Returns the game, the most spells the
/// player was offered to cast, and how many Divinations they cast (choosing as many as
/// offered).
fn wand(roll: u32) -> (TestGame, u32, usize) {
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    let wand = t.battlefield(P0, "Wand of Wonder");
    let divs: Vec<ObjectId> = [P1, P2, P3]
        .iter()
        .map(|p| t.library_top(*p, "Divination"))
        .collect();
    mana(&mut t, P0);
    t.g.dice.loaded.push_back(roll);
    let n = match roll {
        1..=9 => 1,
        10..=19 => 2,
        _ => 3,
    };
    let chosen: Vec<Entity> = divs.iter().take(n).map(|d| Entity::Object(*d)).collect();
    t.answer_choose_moved(P0, &chosen);
    let hand = t.hand_size(P0);
    t.activate(P0, wand, 0, &[]).unwrap();
    t.resolve_all();
    let offered = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                candidates, max, ..
            } if p == P0 && candidates.len() == 3 => Some(max),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let cast = (t.hand_size(P0) - hand) / 2;
    (t, offered, cast)
}

#[test]
fn wand_of_wonder_x_is_the_rows_value_for_the_result() {
    cr!("706.3a", "706.3b");
    supported("Wand of Wonder");
    for (roll, x) in [(1, 1), (9, 1), (10, 2), (19, 2), (20, 3)] {
        let (t, offered, cast) = wand(roll);
        assert_eq!(offered, x, "roll {roll}");
        assert_eq!(cast as u32, x, "roll {roll}: each Divination drew two cards");
        // Each opponent exiled their Divination.
        for p in [P1, P2, P3] {
            assert!(!t.g.player(p).library.iter().any(|c| t.obj_now(*c).chars.name == "Divination"));
        }
    }
}

// ---------------------------------------------------------------------------
// Lae'zel's Acrobatics
// ---------------------------------------------------------------------------

/// Casts Lae'zel's Acrobatics with the d20 showing `roll`; P0 controls Lone Missionary
/// ("When ~ enters, you gain 4 life") and a Soldier token.
fn acrobatics(roll: u32) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let missionary = t.battlefield(P0, "Lone Missionary");
    let acro = t.hand(P0, "Lae'zel's Acrobatics");
    mana(&mut t, P0);
    t.g.dice.loaded.push_back(roll);
    t.cast(P0, acro).go();
    t.resolve_all();
    (t, missionary)
}

#[test]
fn laezels_acrobatics_low_roll_returns_them_at_the_next_end_step() {
    cr!("706.3a", "603.7a");
    supported("Lae'zel's Acrobatics");
    let (mut t, missionary) = acrobatics(5);
    assert!(!t.on_battlefield(missionary));
    assert!(t.in_exile("Lone Missionary"));
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lone Missionary").len(), 1);
    assert_eq!(t.life(P0), 24, "it entered once");
}

#[test]
fn laezels_acrobatics_high_roll_returns_them_then_exiles_them_again() {
    cr!("706.3a", "603.7a", "400.7");
    for roll in [10, 20] {
        let (mut t, _) = acrobatics(roll);
        // It entered (its trigger resolved after the spell) and was exiled again.
        assert!(t.in_exile("Lone Missionary"), "roll {roll}");
        assert!(t.named_on_battlefield("Lone Missionary").is_empty());
        assert_eq!(t.life(P0), 24, "roll {roll}");
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Lone Missionary").len(), 1);
        assert_eq!(t.life(P0), 28, "it entered again at the end step");
    }
}

// ---------------------------------------------------------------------------
// Clay Golem
// ---------------------------------------------------------------------------

#[test]
fn clay_golem_rolls_as_it_activates_and_becomes_monstrous_with_the_result() {
    cr!("118.1", "602.2b", "701.37a");
    supported("Clay Golem");
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Clay Golem");
    let target = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0);
    t.g.dice.loaded.push_back(5);
    t.activate(P0, golem, 0, &[]).unwrap();
    // The die was rolled to pay the cost, before the ability resolves.
    assert_eq!(die_rolls(&t), vec![(P0, 5)]);
    assert_eq!(t.counters(golem, "+1/+1"), 0);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.resolve_all();
    assert_eq!(t.counters(golem, "+1/+1"), 5);
    assert!(t.obj_now(golem).monstrous);
    assert!(!t.on_battlefield(target), "Berserk destroyed it");
}

#[test]
fn clay_golem_can_be_activated_again_in_response() {
    cr!("118.1", "602.2b", "701.37b");
    ruling!(
        "Clay Golem",
        "because rolling the d8 is part of the cost of activating the ability, if you don't like what you rolled"
    );
    ruling!(
        "Clay Golem",
        "You can activate Clay Golem's first ability even after it has become monstrous."
    );
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Clay Golem");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(target)]);
    mana(&mut t, P0);
    mana(&mut t, P0);
    t.g.dice.loaded.extend([1, 7]);
    t.activate(P0, golem, 0, &[]).unwrap();
    t.activate(P0, golem, 0, &[]).unwrap();
    assert_eq!(die_rolls(&t), vec![(P0, 1), (P0, 7)]);
    t.resolve_all();
    // The second activation resolved first (7 counters); the first did nothing.
    assert_eq!(t.counters(golem, "+1/+1"), 7);
    // Once monstrous, it can still be activated: the die is rolled.
    mana(&mut t, P0);
    t.g.dice.loaded.push_back(4);
    t.activate(P0, golem, 0, &[]).unwrap();
    assert_eq!(die_rolls(&t).len(), 3);
    t.resolve_all();
    assert_eq!(t.counters(golem, "+1/+1"), 7);
}

// ---------------------------------------------------------------------------
// Chaos Dragon
// ---------------------------------------------------------------------------

/// Chaos Dragon in a three-player game; the players roll `rolls` (P0, P1, P2) at the
/// beginning of combat. Returns what the Dragon may attack.
fn chaos(rolls: [u32; 3]) -> Vec<Entity> {
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let dragon = t.battlefield(P0, "Chaos Dragon");
    t.g.dice.loaded.extend(rolls);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(
        die_rolls(&t),
        vec![(P0, rolls[0]), (P1, rolls[1]), (P2, rolls[2])],
        "each player rolled, in turn order"
    );
    mtg_engine::combat::attack_options(&t.g)
        .into_iter()
        .find(|(c, _)| *c == dragon)
        .map(|(_, v)| v)
        .unwrap_or_default()
}

#[test]
fn chaos_dragon_cant_attack_opponents_who_had_the_highest_result() {
    cr!("706.2", "508.1c");
    supported("Chaos Dragon");
    ruling!(
        "Chaos Dragon",
        "If you had a higher result than all of your opponents, Chaos Dragon won't have any restrictions"
    );
    let p1 = Entity::Player(P1);
    let p2 = Entity::Player(P2);
    // You had the highest result: no restriction.
    let v = chaos([18, 15, 5]);
    assert!(v.contains(&p1) && v.contains(&p2));
    // P1 had it.
    let v = chaos([10, 15, 5]);
    assert!(!v.contains(&p1) && v.contains(&p2));
    // P1 and P2 tied for it.
    let v = chaos([10, 15, 15]);
    assert!(!v.contains(&p1) && !v.contains(&p2));
}

#[test]
fn chaos_dragon_tie_with_you_still_counts_as_that_opponent_having_the_highest_result() {
    cr!("706.2");
    interp!("dice-highest-result-ties");
    let v = chaos([15, 15, 5]);
    assert!(!v.contains(&Entity::Player(P1)));
    assert!(v.contains(&Entity::Player(P2)));
}

#[test]
fn chaos_dragon_cant_attack_planeswalkers_those_players_control_and_only_this_combat() {
    cr!("508.1c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dragon = t.battlefield(P0, "Chaos Dragon");
    let pw = t.battlefield(P1, "Jace Beleren");
    t.g.dice.loaded.extend([3, 12]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let opts = mtg_engine::combat::attack_options(&t.g);
    let v = opts
        .iter()
        .find(|(c, _)| *c == dragon)
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    assert!(!v.contains(&Entity::Player(P1)));
    assert!(!v.contains(&Entity::Object(pw)));
    // The next turn's combat: a new roll (P0 wins it).
    t.advance_to(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend([19, 2]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let opts = mtg_engine::combat::attack_options(&t.g);
    let v = &opts.iter().find(|(c, _)| *c == dragon).unwrap().1;
    assert!(v.contains(&Entity::Player(P1)) && v.contains(&Entity::Object(pw)));
}

// ---------------------------------------------------------------------------
// Vedalken Squirrel-Whacker
// ---------------------------------------------------------------------------

#[test]
fn squirrel_whacker_rolls_its_base_power_and_toughness_as_it_enters() {
    cr!("614.1c", "706.1", "707.2");
    ruling!(
        "Vedalken Squirrel-Whacker",
        "The two die rolls to set initial base power and toughness are separate and sequential."
    );
    for (p, tough) in [(3, 5), (6, 1)] {
        let mut t = TestGame::new(2);
        let w = t.hand(P0, "Vedalken Squirrel-Whacker");
        mana(&mut t, P0);
        t.g.dice.loaded.extend([p, tough]);
        t.cast(P0, w).go();
        t.resolve_all();
        let w = t.named_on_battlefield("Vedalken Squirrel-Whacker")[0];
        assert_eq!(t.pt(w), (p as i32, tough as i32));
        assert_eq!(die_rolls(&t), vec![(P0, p), (P0, tough)], "two separate rolls");
    }
}

// ---------------------------------------------------------------------------
// Bamboozling Beeble
// ---------------------------------------------------------------------------

/// P1 rolls a d20 (an effect of a P1 ability); returns the result.
fn p1_rolls(t: &mut TestGame) -> i64 {
    let src = t.battlefield(P1, "Grizzly Bears");
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src), P1);
    mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(20), &mut ctx);
    t.g.flush_events();
    ctx.nums[&mtg_engine::dice::RESULT]
}

fn ignore_prompts(t: &TestGame) -> Vec<(PlayerId, Vec<String>)> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a roll to ignore" => Some((p, options)),
            _ => None,
        })
        .collect()
}

#[test]
fn bamboozling_beeble_the_next_roll_has_one_more_die_and_you_choose_the_ignored_one() {
    cr!("706.6", "614.1a");
    supported("Bamboozling Beeble");
    ruling!(
        "Bamboozling Beeble",
        "You choose which roll to ignore after seeing all the results."
    );
    for (ignore, result) in [(0usize, 17), (1, 4)] {
        let mut t = TestGame::new(2);
        let beeble = t.battlefield(P0, "Bamboozling Beeble");
        mana(&mut t, P0);
        t.activate(P0, beeble, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        t.g.dice.loaded.extend([4, 17]);
        t.answer(P0, DecisionKind::Option, Answer::Index(ignore));
        assert_eq!(p1_rolls(&mut t), result);
        // P0 chose, seeing both results; the ignored roll never happened.
        let asked = ignore_prompts(&t);
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].0, P0);
        assert_eq!(asked[0].1.len(), 2);
        assert_eq!(die_rolls(&t), vec![(P1, result as u32)]);
        // Used up: the next roll is a single die.
        t.g.dice.loaded.extend([9]);
        assert_eq!(p1_rolls(&mut t), 9);
        assert_eq!(ignore_prompts(&t).len(), 1);
    }
}

#[test]
fn bamboozling_beeble_affects_only_that_player_and_only_this_turn() {
    cr!("706.6", "614.1a");
    let mut t = TestGame::new(2);
    let beeble = t.battlefield(P0, "Bamboozling Beeble");
    mana(&mut t, P0);
    t.activate(P0, beeble, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // P0's own roll: one die.
    let mut ctx = mtg_engine::eval::Ctx::new(Some(beeble), P0);
    t.g.dice.loaded.extend([6]);
    mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(20), &mut ctx);
    assert_eq!(ctx.nums[&mtg_engine::dice::RESULT], 6);
    assert!(ignore_prompts(&t).is_empty());
    // Next turn: gone.
    t.advance_to(P1, Step::Upkeep);
    t.g.dice.loaded.extend([12]);
    assert_eq!(p1_rolls(&mut t), 12);
    assert!(ignore_prompts(&t).is_empty());
}

#[test]
fn two_bamboozling_beebles_both_apply_to_the_next_roll() {
    cr!("706.6", "616.1");
    interp!("next-roll-extra-die-same-roll");
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P0, "Bamboozling Beeble");
    let b2 = t.battlefield(P0, "Bamboozling Beeble");
    mana(&mut t, P0);
    t.activate(P0, b1, 0, &[Entity::Player(P1)]).unwrap();
    t.activate(P0, b2, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // Three dice: P0 ignores two of them, one at a time, seeing what's left.
    t.g.dice.loaded.extend([8, 2, 15]);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert_eq!(p1_rolls(&mut t), 2);
    let asked = ignore_prompts(&t);
    assert_eq!(asked.len(), 2);
    assert_eq!((asked[0].1.len(), asked[1].1.len()), (3, 2));
    // Both used up.
    t.g.dice.loaded.extend([5]);
    assert_eq!(p1_rolls(&mut t), 5);
}

// ---------------------------------------------------------------------------
// Mr. House, President and CEO
// ---------------------------------------------------------------------------

fn treasure(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure token");
    let tc = mtg_engine::replacement::TokenCreate {
        chars: mtg_engine::tokens::token_characteristics(&spec),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    t.g.create_tokens(p, tc, 1, None)[0]
}

fn subtype_count(t: &TestGame, p: PlayerId, st: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype(st))
        .count()
}

/// Mr. House activated with `treasures` Treasures and 4 − `treasures` Mountains (all used
/// to pay {4}); the dice show `dice`.
fn mr_house(treasures: usize, dice: &[u32]) -> TestGame {
    let mut t = TestGame::new(2);
    let house = t.battlefield(P0, "Mr. House, President and CEO");
    t.lands(P0, "Mountain", 4 - treasures);
    for _ in 0..treasures {
        treasure(&mut t, P0);
    }
    t.g.dice.loaded.extend(dice.iter().copied());
    t.activate(P0, house, 0, &[]).unwrap();
    t.resolve_all();
    t
}

#[test]
fn mr_house_rolls_an_extra_die_for_each_mana_from_treasures() {
    cr!("706.1", "106.3", "602.2b");
    supported("Mr. House, President and CEO");
    ruling!(
        "Mr. House, President and CEO",
        "Abilities that trigger whenever you roll dice or whenever you roll a certain result, such as Mr. House's first ability, will trigger as appropriate."
    );
    // Two Treasures: three dice. A 4 makes a Robot; a 6 a Robot and a Treasure.
    let t = mr_house(2, &[4, 6, 2]);
    assert_eq!(die_rolls(&t), vec![(P0, 4), (P0, 6), (P0, 2)]);
    assert_eq!(subtype_count(&t, P0, "Robot"), 2);
    assert_eq!(subtype_count(&t, P0, "Treasure"), 1, "the two spent were sacrificed");
    // No Treasure: one die.
    let t = mr_house(0, &[5, 6]);
    assert_eq!(die_rolls(&t), vec![(P0, 5)]);
    assert_eq!(subtype_count(&t, P0, "Robot"), 1);
}
