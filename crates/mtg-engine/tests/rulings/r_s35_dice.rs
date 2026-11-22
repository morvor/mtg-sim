//! Rulings batch S35 — rolling dice (CR 706): an N-sided die has the outcomes 1 to N, the
//! ability that rolls says what to do with the result (often with a results table), and
//! the "result" is the natural result after modifications.

use crate::r_s01_common::{attack_with, supported, with_subtype};
use crate::r_s35_common::{die_rolls, load_dice};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use std::collections::BTreeSet;

/// A game whose random number generator starts from `seed`.
fn seeded(seed: u64) -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            seed,
            ..Default::default()
        },
    )
}

/// P0's Hoarding Ogre attacks P1 ("Whenever this creature attacks, roll a d20. 1—9 |
/// Create a Treasure token. 10—19 | Create two Treasure tokens. 20 | Create three
/// Treasure tokens."); returns the Treasures P0 has afterwards.
fn ogre_attacks(t: &mut TestGame) -> usize {
    let ogre = t.battlefield(P0, "Hoarding Ogre");
    attack_with(t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    with_subtype(t, P0, "Treasure").len()
}

/// P0 casts Clowning Around ("Create two 1/1 white Clown Robot artifact creature tokens,
/// then roll a six-sided die. If the result is equal to or less than the number of Robots
/// you control, create a 1/1 white Clown Robot artifact creature token."); returns the
/// Robots P0 controls afterwards.
fn clowning_around(t: &mut TestGame) -> usize {
    t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Clowning Around");
    t.cast(P0, spell).go();
    t.resolve_all();
    with_subtype(t, P0, "Robot").len()
}

#[test]
fn a_d20_is_a_twenty_sided_die() {
    cr!("706.1", "706.1a");
    ruling!(
        "Hoarding Ogre",
        "Dice are identified by the number of faces each one has. For example, a d20 is a twenty-sided die."
    );
    supported("Hoarding Ogre");
    let mut seen = BTreeSet::new();
    for seed in 0..16 {
        let mut t = seeded(seed);
        let treasures = ogre_attacks(&mut t);
        let rolls = die_rolls(&t);
        assert_eq!(rolls.len(), 1);
        let (player, sides, natural, result) = rolls[0];
        assert_eq!((player, sides), (P0, 20));
        assert!((1..=20).contains(&natural));
        assert_eq!(natural, result);
        let expected = match result {
            1..=9 => 1,
            10..=19 => 2,
            _ => 3,
        };
        assert_eq!(treasures, expected, "result {result}");
        seen.insert(natural);
    }
    // The outcomes vary over the whole range, not just a six-sided die's.
    assert!(seen.len() >= 8, "{seen:?}");
    assert!(seen.iter().any(|r| *r > 6), "{seen:?}");
}

#[test]
fn the_ability_says_what_to_do_with_the_result_in_a_results_table() {
    cr!("706.3", "706.3a", "706.3b");
    ruling!(
        "Hoarding Ogre",
        "An ability that tells you to roll a die will also specify what to do with the result of that roll. Most often, this is in the form of a \"results table\" in the card text."
    );
    supported("Hoarding Ogre");
    for (natural, treasures) in [(1, 1), (9, 1), (10, 2), (19, 2), (20, 3)] {
        let mut t = TestGame::new(2);
        load_dice(&mut t, &[natural]);
        assert_eq!(ogre_attacks(&mut t), treasures, "natural {natural}");
    }
}

#[test]
fn chaos_channelers_results_table() {
    cr!("706.3", "706.3a", "706.3b");
    ruling!(
        "Chaos Channeler",
        "An ability that tells you to roll a die will also specify what to do with the result of that roll. Most often, this is in the form of a “results table” in the card text."
    );
    supported("Chaos Channeler");
    // "Whenever this creature attacks, roll a d20. 1—9 | Exile the top card of your
    // library. You may play it this turn. 10—19 | Exile the top two cards of your library.
    // You may play them this turn. 20 | Exile the top three cards of your library. You may
    // play them this turn."
    for (natural, exiled) in [(4, 1), (15, 2), (20, 3)] {
        let mut t = TestGame::new(2);
        let top = crate::r_s01_common::stack_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Forest", "Island", "Mountain"],
        );
        load_dice(&mut t, &[natural]);
        let channeler = t.battlefield(P0, "Chaos Channeler");
        attack_with(&mut t, &[(channeler, Entity::Player(P1))]);
        t.resolve_all();
        for (i, card) in top.iter().enumerate() {
            let in_exile = t.zone(t.g.current(*card)) == Zone::Exile;
            assert_eq!(in_exile, i < exiled, "natural {natural}, card {i}");
        }
    }
}

#[test]
fn the_result_is_after_modifications_and_the_natural_result_before() {
    cr!("706.2", "706.2a");
    ruling!(
        "Hoarding Ogre",
        "Some effects may modify the result of a die roll. This may be part of the instruction to roll a die or it may come from other cards. Anything that references the \"result\" of a die roll is looking for the result after these modifications. Anything that is looking for the \"natural result\" is looking for the number shown on the face of the die before these modifications."
    );
    supported("Snickering Squirrel");
    supported("Netherese Puzzle-Ward");
    // Snickering Squirrel: "You may tap this creature to increase the result of a die any
    // player rolled by 1." Netherese Puzzle-Ward: "Whenever you roll a die's highest
    // natural result, draw a card."
    // A natural 19 increased to 20: the results table's "20" row applies, but it isn't
    // the die's highest natural result.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snickering Squirrel");
    t.battlefield(P0, "Netherese Puzzle-Ward");
    t.answer_yes(P0, true);
    load_dice(&mut t, &[19]);
    let hand = t.hand_size(P0);
    assert_eq!(ogre_attacks(&mut t), 3);
    assert_eq!(die_rolls(&t), vec![(P0, 20, 19, 20)]);
    assert_eq!(t.hand_size(P0), hand);
    // A natural 20 (not increased) is: three Treasures and a card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snickering Squirrel");
    t.battlefield(P0, "Netherese Puzzle-Ward");
    t.answer_yes(P0, false);
    load_dice(&mut t, &[20]);
    let hand = t.hand_size(P0);
    assert_eq!(ogre_attacks(&mut t), 3);
    assert_eq!(die_rolls(&t), vec![(P0, 20, 20, 20)]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn only_a_roll_the_game_instructs_counts_as_rolling_a_die() {
    cr!("706.1", "705.1");
    ruling!(
        "Clowning Around",
        "Something in the game must tell you to roll a die. If you roll a die for any other reason (to simulate a coin flip, to choose pizza toppings, to create alternate timelines), that roll doesn't count."
    );
    supported("Clowning Around");
    supported("Brazen Dwarf");
    supported("Molten Birth");
    // Brazen Dwarf: "Whenever you roll one or more dice, this creature deals 1 damage to
    // each opponent." Molten Birth flips a coin: that's not a die roll, however the coin
    // is simulated.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brazen Dwarf");
    t.lands(P0, "Mountain", 3);
    let birth = t.hand(P0, "Molten Birth");
    t.cast(P0, birth).go();
    t.resolve_all();
    assert!(die_rolls(&t).is_empty());
    assert_eq!(t.life(P1), 20);
    // Clowning Around instructs P0 to roll a six-sided die: that roll counts.
    clowning_around(&mut t);
    assert_eq!(die_rolls(&t).len(), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_six_sided_die_has_six_equally_likely_outcomes() {
    cr!("706.1", "706.1a", "706.1b");
    ruling!(
        "Clowning Around",
        "Each die is identified by the number of faces it has. A six-sided die is a die with six equally likely outcomes: 1, 2, 3, 4, 5, and 6."
    );
    supported("Clowning Around");
    let mut seen = BTreeSet::new();
    for seed in 0..40 {
        let mut t = seeded(seed);
        let robots = clowning_around(&mut t);
        let rolls = die_rolls(&t);
        assert_eq!(rolls.len(), 1);
        let (_, sides, natural, result) = rolls[0];
        assert_eq!(sides, 6);
        assert!((1..=6).contains(&natural));
        // Two Robots were created: a 1 or a 2 creates a third.
        assert_eq!(robots, if result <= 2 { 3 } else { 2 }, "result {result}");
        seen.insert(natural);
    }
    assert_eq!(seen, (1..=6).collect());
}

#[test]
fn a_modified_result_can_be_a_number_a_die_cant_show() {
    cr!("706.2");
    ruling!(
        "Clowning Around",
        "Results can be numbers not ordinarily possible on a six-sided die. Spells like Scooch can change the result to 0 or 7, for example."
    );
    supported("Snickering Squirrel");
    // Two earlier Clowning Arounds (natural 6s) made four Robots. A third one makes six:
    // a natural 6 is "equal to or less than" six; a 6 increased to 7 isn't.
    for (squirrel, robots) in [(false, 7), (true, 6)] {
        let mut t = TestGame::new(2);
        load_dice(&mut t, &[6, 6, 6]);
        clowning_around(&mut t);
        assert_eq!(clowning_around(&mut t), 4);
        if squirrel {
            t.battlefield(P0, "Snickering Squirrel");
            t.answer_yes(P0, true);
        }
        assert_eq!(clowning_around(&mut t), robots);
        let last = *die_rolls(&t).last().unwrap();
        assert_eq!(last, (P0, 6, 6, if squirrel { 7 } else { 6 }));
    }
}

#[test]
fn the_result_is_the_result_after_modifications() {
    cr!("706.2", "706.2a");
    ruling!(
        "Clowning Around",
        "Some effects may modify the result of a die roll. This may be part of the instruction to roll a die, or it may come from other cards. Anything that references the \"result\" of a die roll is looking for the result after these modifications."
    );
    // A natural 2 with two Robots: a third Robot. Increased to 3 by Snickering Squirrel
    // (another card), the result is 3: no third Robot.
    for (squirrel, robots) in [(false, 3), (true, 2)] {
        let mut t = TestGame::new(2);
        if squirrel {
            t.battlefield(P0, "Snickering Squirrel");
            t.answer_yes(P0, true);
        }
        load_dice(&mut t, &[2]);
        assert_eq!(clowning_around(&mut t), robots, "squirrel: {squirrel}");
        assert_eq!(die_rolls(&t)[0].3, if squirrel { 3 } else { 2 });
    }
}
