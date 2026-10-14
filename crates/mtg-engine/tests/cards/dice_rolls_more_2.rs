//! More die rolls and coin flips (CR 705, 706): "that many" after a die-roll trigger,
//! results tables with flavor words and "those tokens", die-roll modifiers with costs,
//! counts of one result, flips with a limit, and names of tokens.

use mtg_engine::decision::Answer;
use mtg_engine::dice::DiceIgnore;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// `p` rolls one die with the given natural result (as if for another card's ability).
fn roll(t: &mut TestGame, p: PlayerId, sides: u32, natural: u32) -> i64 {
    t.g.dice.loaded.push_back(natural);
    let d = mtg_engine::dice::roll_dice(&mut t.g, p, 1, sides, DiceIgnore::None, 0, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    d[0].result()
}

fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

#[test]
fn that_many_after_a_die_roll_trigger_is_the_result() {
    cr!("706.4");
    compiles("Atomwheel Acrobats");
    // "Whenever you roll a 1 or 2, put that many +1/+1 counters on this creature."
    let mut t = TestGame::new(2);
    let acrobats = t.battlefield(P0, "Atomwheel Acrobats");
    roll(&mut t, P0, 6, 2);
    assert_eq!(t.counters(acrobats, counters::PLUS1), 2);
    roll(&mut t, P0, 6, 1);
    assert_eq!(t.counters(acrobats, counters::PLUS1), 3);
    roll(&mut t, P0, 6, 3);
    assert_eq!(t.counters(acrobats, counters::PLUS1), 3);
}

#[test]
fn instead_create_that_token_and_a_treasure() {
    cr!("706.4");
    // "Whenever you roll a 4 or higher, create a 3/3 colorless Robot artifact creature
    // token. If you rolled 6 or higher, instead create that token and a Treasure token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mr. House, President and CEO");
    roll(&mut t, P0, 6, 3);
    assert_eq!(tokens(&t, P0, "Robot"), 0);
    roll(&mut t, P0, 6, 4);
    assert_eq!((tokens(&t, P0, "Robot"), tokens(&t, P0, "Treasure")), (1, 0));
    roll(&mut t, P0, 6, 6);
    assert_eq!((tokens(&t, P0, "Robot"), tokens(&t, P0, "Treasure")), (2, 1));
}

#[test]
fn you_may_have_it_become_an_x_x_creature() {
    cr!("706.4");
    compiles("Ebony Fly");
    // "{4}: Roll a d6. Until end of turn, you may have this artifact become an X/X Insect
    // artifact creature with flying, where X is the result."
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let fly = t.battlefield(P0, "Ebony Fly");
        t.g.objects[fly.0 as usize].tapped = false;
        t.lands(P0, "Plains", 4);
        t.set_step(P0, Step::PrecombatMain);
        t.g.dice.loaded.push_back(5);
        t.answer_yes(P0, yes);
        t.activate(P0, fly, 1, &[]).unwrap();
        t.resolve_all();
        let o = t.obj_now(fly);
        assert_eq!(o.chars.is(mtg_engine::types::CardType::Creature), yes);
        if yes {
            assert_eq!(t.pt(fly), (5, 5));
        }
    }
}

#[test]
fn the_result_is_greater_than_the_damage_dealt() {
    cr!("706.4");
    compiles("Sword of Hours");
    // "Whenever equipped creature deals combat damage, roll a d12. If the result is greater
    // than the damage dealt or the result is 12, double the number of +1/+1 counters on
    // that creature." ("Whenever equipped creature attacks, put a +1/+1 counter on it.")
    for (natural, doubled) in [(4, true), (3, false), (12, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let sword = t.battlefield(P0, "Sword of Hours");
        t.g.attach(sword, Entity::Object(bears));
        t.g.dice.loaded.push_back(natural);
        t.set_step(P0, Step::PrecombatMain);
        t.attack(&[(bears, Entity::Player(P1))], &[]);
        // A 3/3 dealt 3 damage.
        assert_eq!(t.life(P1), 17);
        let want = if doubled { 2 } else { 1 };
        assert_eq!(t.counters(bears, counters::PLUS1), want, "a roll of {natural}");
    }
}

#[test]
fn results_table_rows_with_those_tokens_and_flavor_words() {
    cr!("706.3a");
    compiles("Swarming Goblins");
    compiles("Treasure Chest");
    // "When this creature enters, roll a d20. 1—9 | Create a 1/1 red Goblin creature token.
    // 10—19 | Create two of those tokens. 20 | Create three of those tokens."
    for (natural, n) in [(5, 1), (15, 2), (20, 3)] {
        let mut t = TestGame::new(2);
        t.g.dice.loaded.push_back(natural);
        t.enter(P0, "Swarming Goblins");
        t.resolve_all();
        let goblins: Vec<(i32, i32)> = t
            .g
            .permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Goblin"))
            .map(|o| (o.power(), o.toughness()))
            .collect();
        assert_eq!(goblins, vec![(1, 1); n], "a roll of {natural}");
    }
    // "{4}, Sacrifice this artifact: Roll a d20. 1 | Trapped! — You lose 3 life."
    let mut t = TestGame::new(2);
    let chest = t.battlefield(P0, "Treasure Chest");
    t.lands(P0, "Plains", 4);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.push_back(1);
    t.activate(P0, chest, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn after_you_roll_a_die_you_may_pay_to_change_the_result() {
    cr!("706.2a");
    compiles("Night Shift of the Living Dead");
    compiles("Xenosquirrels");
    ruling!(
        "Night Shift of the Living Dead",
        "The first ability can affect only one die"
    );
    // "After you roll a die, you may pay 1 life. If you do, increase or decrease the result
    // by 1. Do this only once each turn." "Whenever you roll a 6, create a 2/2 black Zombie
    // Employee creature token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Night Shift of the Living Dead");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    // Increase (the first option).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert_eq!(roll(&mut t, P0, 6, 5), 6);
    assert_eq!(t.life(P0), 19);
    assert_eq!(tokens(&t, P0, "Zombie"), 1);
    // Only once each turn.
    assert_eq!(roll(&mut t, P0, 6, 5), 5);
    assert_eq!(t.life(P0), 19);
    // Xenosquirrels: "After you roll a die, you may remove a +1/+1 counter from this
    // creature. If you do, increase or decrease the result by 1."
    let mut t = TestGame::new(2);
    let squirrels = t.battlefield(P0, "Xenosquirrels");
    t.g.objects[squirrels.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 3);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert_eq!(roll(&mut t, P0, 6, 3), 2);
    assert_eq!(t.counters(squirrels, counters::PLUS1), 2);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert_eq!(roll(&mut t, P0, 6, 3), 4);
    assert_eq!(t.counters(squirrels, counters::PLUS1), 1);
    ruling!("Xenosquirrels", "You can change a result to something normally not possible");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert_eq!(roll(&mut t, P0, 6, 6), 7);
    assert_eq!(t.counters(squirrels, counters::PLUS1), 0);
}

#[test]
fn rolled_a_result_exactly_n_times() {
    cr!("706.1a");
    compiles("Luck Bobblehead");
    ruling!(
        "Luck Bobblehead",
        "You win the game only if you rolled 6 exactly seven times while resolving a single instance"
    );
    // "{1}, {T}: Roll X six-sided dice, where X is the number of Bobbleheads you control.
    // Create a tapped Treasure token for each even result. If you rolled 6 exactly seven
    // times, you win the game."
    let mut t = TestGame::new(2);
    let bobbles: Vec<ObjectId> = (0..7)
        .map(|_| t.battlefield(P0, "Luck Bobblehead"))
        .collect();
    t.lands(P0, "Plains", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend([6, 6, 6, 6, 6, 6, 5]);
    t.activate(P0, bobbles[0], 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Treasure"), 6);
    assert!(t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Treasure"))
        .all(|o| o.tapped));
    assert!(!t.has_lost(P1));
    t.g.dice.loaded.extend([6; 7]);
    t.activate(P0, bobbles[1], 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn flip_a_coin_that_many_times_or_until_you_lose_a_flip() {
    cr!("705.2");
    compiles("Squee's Revenge");
    // "Choose a number. Flip a coin that many times or until you lose a flip, whichever
    // comes first. If you win all the flips, draw two cards for each flip."
    for (coins, drawn) in [(vec![true, true, true], 6), (vec![true, false, true], 0)] {
        let mut t = TestGame::new(2);
        for _ in 0..10 {
            t.library_top(P0, "Island");
        }
        t.lands(P0, "Volcanic Island", 3);
        t.set_step(P0, Step::PrecombatMain);
        t.answer(P0, DecisionKind::Number, Answer::Number(3));
        t.g.dice.loaded_coins.extend(coins.clone());
        let spell = t.hand(P0, "Squee's Revenge");
        let before = t.hand_size(P0);
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0) + 1, before + drawn, "{coins:?}");
        // A lost flip ends the flipping.
        let flips = if coins[1] { 3 } else { 2 };
        assert_eq!(t.g.dice.loaded_coins.len(), 3 - flips);
    }
}

#[test]
fn tokens_named_and_permanents_named() {
    cr!("201.2");
    compiles("Wirefly Hive");
    // "{3}, {T}: Flip a coin. If you win the flip, create a 2/2 colorless Insect artifact
    // creature token with flying named Wirefly. If you lose the flip, destroy all
    // permanents named Wirefly."
    let mut t = TestGame::new(2);
    let hive = t.battlefield(P0, "Wirefly Hive");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 6);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded_coins.push_back(true);
    t.activate(P0, hive, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Wirefly").len(), 1);
    // Untap and lose the next flip.
    let hive_now = t.g.current(hive);
    t.g.objects[hive_now.0 as usize].tapped = false;
    t.g.dice.loaded_coins.push_back(false);
    t.activate(P0, hive, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.named_on_battlefield("Wirefly").is_empty());
    assert!(t.on_battlefield(bears));
}

#[test]
fn each_creature_not_chosen_this_way() {
    cr!("706.3a");
    compiles("Thunderwave");
    // "Roll a d20. ... 10—19 | You may choose a creature. Thunderwave deals 3 damage to each
    // creature not chosen this way."
    for choose in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Mountain", 4);
        t.set_step(P0, Step::PrecombatMain);
        t.g.dice.loaded.push_back(15);
        t.answer_yes(P0, choose);
        t.answer_choose(P0, &[Entity::Object(bears)]);
        let spell = t.hand(P0, "Thunderwave");
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), choose);
        assert!(!t.on_battlefield(giant));
    }
}
