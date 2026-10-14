//! Die rolls (CR 706) compiled with `oracle/patterns/dice_coin_ext.rs` and
//! `r705_706_dice.rs`: "roll two d12 and choose one result", "for each even result",
//! "the result minus 1", "a natural 20", "your third die each turn", dice rolled this
//! turn, and "if the roll was 4 or higher".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::dice::DiceIgnore;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
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
fn roll(t: &mut TestGame, p: PlayerId, sides: u32, natural: u32) {
    t.g.dice.loaded.push_back(natural);
    mtg_engine::dice::roll_dice(&mut t.g, p, 1, sides, DiceIgnore::None, 0, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
}

fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

#[test]
fn roll_two_dice_and_choose_one_result() {
    cr!("706.1a", "706.4");
    for name in [
        "Reckless Endeavor",
        "Wild Endeavor",
        "Valiant Endeavor",
        "Arcane Endeavor",
        "Grave Endeavor",
    ] {
        compiles(name);
    }
    // "Roll two d12 and choose one result. Reckless Endeavor deals damage equal to that
    // result to each creature. Then create a number of Treasure tokens equal to the other
    // result."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 7);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend([2, 9]);
    // Choose the 2 (the first result).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let spell = t.hand(P0, "Reckless Endeavor");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose one result")));
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    assert_eq!(tokens(&t, P0, "Treasure"), 9);
}

#[test]
fn for_each_even_or_odd_result() {
    cr!("706.4");
    compiles("Clown Car");
    compiles("Attempted Murder");
    // "When this Vehicle enters, roll X six-sided dice. For each odd result, create a 1/1
    // white Clown Robot artifact creature token. For each even result, put a +1/+1 counter
    // on this Vehicle."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend([1, 4, 5]);
    let car = t.hand(P0, "Clown Car");
    t.cast(P0, car).x(3).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Robot"), 2);
    assert_eq!(t.counters(car, counters::PLUS1), 1);
}

#[test]
fn the_result_minus_one() {
    cr!("706.4");
    compiles("Velukan Dragon");
    // "Whenever this creature attacks or blocks, roll a six-sided die. This creature gets
    // +X/+0 until end of turn, where X is the result minus 1."
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Velukan Dragon");
    t.g.dice.loaded.push_back(4);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert_eq!(t.pt(dragon), (8, 5));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn scry_a_number_of_cards_equal_to_the_result() {
    cr!("706.4");
    compiles("Boing!");
    // "Return target creature to its owner's hand, then roll a six-sided die. If the
    // result is 3 or less, scry a number of cards equal to the result."
    for (natural, scried) in [(2, Some(2)), (5, None)] {
        let mut t = TestGame::new(2);
        for _ in 0..4 {
            t.library_top(P0, "Island");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Island", 2);
        t.set_step(P0, Step::PrecombatMain);
        t.g.dice.loaded.push_back(natural);
        let spell = t.hand(P0, "Boing!");
        t.cast(P0, spell).target(Entity::Object(bears)).go();
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Hand(P1));
        let scry = t.asked().into_iter().find_map(|(_, d)| match d {
            Decision::Scry { cards } => Some(cards.len()),
            _ => None,
        });
        assert_eq!(scry, scried, "a roll of {natural}");
    }
}

#[test]
fn when_you_roll_a_natural_20() {
    cr!("706.2");
    compiles("Critical Hit");
    // "When you roll a natural 20, return this card from your graveyard to your hand."
    let mut t = TestGame::new(2);
    let hit = t.graveyard(P0, "Critical Hit");
    roll(&mut t, P0, 20, 19);
    assert_eq!(t.zone(hit), Zone::Graveyard(P0));
    // An opponent's natural 20 isn't yours.
    roll(&mut t, P1, 20, 20);
    assert_eq!(t.zone(hit), Zone::Graveyard(P0));
    roll(&mut t, P0, 20, 20);
    assert_eq!(t.zone(hit), Zone::Hand(P0));
}

#[test]
fn your_third_die_each_turn_and_dice_rolled_this_turn() {
    cr!("706.1");
    compiles("Resolute Veggiesaur");
    compiles("The Space Family Goblinson");
    // "Whenever you roll your third die each turn, put a +1/+1 counter on this creature."
    // "The Space Family Goblinson has trample as long as you've rolled three or more dice
    // this turn."
    let mut t = TestGame::new(2);
    let veggie = t.battlefield(P0, "Resolute Veggiesaur");
    let goblinson = t.battlefield(P0, "The Space Family Goblinson");
    t.set_step(P0, Step::PrecombatMain);
    roll(&mut t, P0, 6, 1);
    roll(&mut t, P1, 6, 1);
    roll(&mut t, P0, 6, 1);
    assert_eq!(t.counters(veggie, counters::PLUS1), 0);
    assert!(!t.obj_now(goblinson).has_keyword(KeywordKind::Trample));
    roll(&mut t, P0, 6, 1);
    assert_eq!(t.counters(veggie, counters::PLUS1), 1);
    assert!(t.obj_now(goblinson).has_keyword(KeywordKind::Trample));
    roll(&mut t, P0, 6, 1);
    assert_eq!(t.counters(veggie, counters::PLUS1), 1);
    // "Whenever you roll a die, put a +1/+1 counter on The Space Family Goblinson."
    assert_eq!(t.counters(goblinson, counters::PLUS1), 4);
}

#[test]
fn the_third_die_among_dice_rolled_at_once() {
    cr!("706.1");
    ruling!(
        "Resolute Veggiesaur",
        "If you roll multiple dice at the same time, and one of them is your third die roll that turn"
    );
    let mut t = TestGame::new(2);
    let veggie = t.battlefield(P0, "Resolute Veggiesaur");
    t.g.dice.loaded.extend([2, 3, 4, 5]);
    mtg_engine::dice::roll_dice(&mut t.g, P0, 4, 6, DiceIgnore::None, 0, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(veggie, counters::PLUS1), 1);
}

#[test]
fn if_the_roll_was_n_or_higher() {
    cr!("706.4");
    compiles("Monoxa, Midway Manager");
    // "Whenever you roll a 3 or higher, Monoxa gains first strike until end of turn. If
    // the roll was 4 or higher, it gains menace until end of turn. If the roll was 5 or
    // higher, it gains lifelink until end of turn."
    for (natural, kws) in [
        (2, [false, false, false]),
        (4, [true, true, false]),
        (6, [true, true, true]),
    ] {
        let mut t = TestGame::new(2);
        let monoxa = t.battlefield(P0, "Monoxa, Midway Manager");
        roll(&mut t, P0, 6, natural);
        let o = t.obj_now(monoxa);
        let got = [
            o.has_keyword(KeywordKind::FirstStrike),
            o.has_keyword(KeywordKind::Menace),
            o.has_keyword(KeywordKind::Lifelink),
        ];
        assert_eq!(got, kws, "a roll of {natural}");
    }
}

#[test]
fn enters_with_counters_equal_to_the_total_of_the_results() {
    cr!("706.4");
    compiles("Neverwinter Hydra");
    // "As this creature enters, roll X d6. It enters with a number of +1/+1 counters on it
    // equal to the total of those results."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend([3, 5]);
    let hydra = t.hand(P0, "Neverwinter Hydra");
    t.cast(P0, hydra).x(2).go();
    t.resolve_all();
    assert!(t.on_battlefield(hydra));
    assert_eq!(t.counters(hydra, counters::PLUS1), 8);
}
