//! CR 601.2c: "If the spell has a variable number of targets, the player announces how
//! many targets they will choose before they announce those targets. In some cases, the
//! number of targets will be defined by the spell's text." — "X target creatures" is
//! exactly X of them, and a multikicker spell's "choose target creature, then choose
//! another target creature for each time this spell was kicked" is exactly one more than
//! the number of times it was kicked (CR 702.33c, 702.33d), both announced before the
//! targets are chosen (CR 601.2b).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

/// The (min, max) of every target choice asked since `from`.
fn target_bounds(t: &TestGame, from: usize) -> Vec<(u32, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .collect()
}

#[test]
fn x_target_creatures_is_exactly_x() {
    cr!("601.2c", "601.2b", "115.1");
    // Thrive ({2}{G}, "Put a +1/+1 counter on each of X target creatures"), X = 2: two
    // targets must be chosen; an answer naming one isn't legal and two are chosen.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 5);
    let thrive = t.hand(P0, "Thrive");
    let from = t.asked().len();
    let spell = t.cast(P0, thrive).x(2).target(bears).go();
    assert_eq!(target_bounds(&t, from), vec![(2, 2)]);
    let chosen = targets_of(&t, spell);
    assert_eq!(chosen.len(), 2);
    assert!(chosen.contains(&Entity::Object(bears)));
    t.resolve();
    let counters: u32 = [bears, giant, elves]
        .iter()
        .map(|c| t.counters(*c, "+1/+1"))
        .sum();
    assert_eq!(counters, 2);
    assert_eq!(t.counters(bears, "+1/+1"), 1);

    // With X greater than the number of creatures that can be targeted, the spell can't
    // be cast: X = 4 with three creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 7);
    let thrive = t.hand(P0, "Thrive");
    assert!(t.cast(P0, thrive).x(4).try_go().is_err());
    assert!(t.in_hand(P0, "Thrive"));

    // X = 0: no targets at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let thrive = t.hand(P0, "Thrive");
    let spell = t.cast(P0, thrive).x(0).go();
    assert!(targets_of(&t, spell).is_empty());
}

#[test]
fn multikicker_targets_are_one_more_than_the_kicks() {
    cr!("601.2c", "601.2b", "702.33c", "702.33d");
    // Strength of the Tajuru ({X}{G}{G}, multikicker {1}) kicked twice: exactly three
    // targets; an answer naming two isn't legal.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Llanowar Elves");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::G, 6);
    let spell = t.hand(P0, "Strength of the Tajuru");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    let from = t.asked().len();
    let id = t
        .cast(P0, spell)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    assert_eq!(target_bounds(&t, from), vec![(3, 3)]);
    assert_eq!(targets_of(&t, id).len(), 3);
    t.resolve();
    for x in [a, b, c] {
        assert_eq!(t.counters(x, "+1/+1"), 2);
    }

    // Kicked more times than there are creatures to target: it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::G, 6);
    let spell = t.hand(P0, "Strength of the Tajuru");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    assert!(t.cast(P0, spell).x(1).try_go().is_err());
}
