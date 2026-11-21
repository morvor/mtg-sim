//! Rulings batch P100 — damage with Obosh, the Preypiercer (sources with an odd mana value
//! deal double damage): combat damage is assigned unmodified (CR 510.1, 702.19b), divided
//! damage is divided unmodified (601.2d), and the affected player orders the replacement
//! and prevention effects (616.1); Bite Down on Crime with an illegal target (608.2b).

use crate::r_p100_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn obosh_doubles_trample_damage_after_it_is_assigned_unmodified() {
    cr!("614.1a", "702.19b", "510.1a", "510.2");
    interp!("default-damage-assignment");
    ruling!(
        "Obosh, the Preypiercer",
        "If a creature with trample you control would deal combat damage to a blocking creature while you control Obosh, you must assign its unmodified damage. For example, a 3/3 creature with trample blocked by a 2/2 creature can have 1 damage assigned to the defending player. It will then deal 4 damage to the blocking creature and 2 damage to the defending player."
    );
    supported("Obosh, the Preypiercer");
    supported("Jhovall Rider");
    // Jhovall Rider ({4}{W}, a 3/3 with trample) blocked by Grizzly Bears: 2 must be
    // assigned to the Bears; 1 may go to P1. The Bears is dealt 4, P1 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obosh, the Preypiercer");
    let rider = t.battlefield(P0, "Jhovall Rider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    t.attack(&[(rider, pl(P1))], &[(bears, rider)]);
    let asked: Vec<Decision> = asked_of(&t, P0, from)
        .into_iter()
        .filter(|d| matches!(d, Decision::AssignCombatDamage { .. }))
        .collect();
    // (The decision is kept as asked: 3 unmodified damage, 2 lethal to the Bears.)
    assert!(
        matches!(asked.first(), Some(Decision::AssignCombatDamage { amount: 3, lethal, .. }) if lethal[0] == 2),
        "{asked:?}"
    );
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18, "1 assigned, doubled");
    // Assigning only 1 to the Bears (not lethal) is illegal: the engine falls back to its
    // default assignment (lethal 2 to the Bears, the rest to the player; register
    // default-damage-assignment).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obosh, the Preypiercer");
    let rider = t.battlefield(P0, "Jhovall Rider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 2]));
    t.attack(&[(rider, pl(P1))], &[(bears, rider)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn obosh_doubles_divided_damage_after_the_division() {
    cr!("614.1a", "601.2d", "603.3d", "608.2c");
    ruling!(
        "Obosh, the Preypiercer",
        "If an effect such as that of Ravenous Gigantotherium asks you to divide damage among targets, you must divide the unmodified damage before doubling it."
    );
    supported("Obosh, the Preypiercer");
    supported("Ravenous Gigantotherium");
    // A Ravenous Gigantotherium ({5}{G}{G}, 7; a 3/3 devouring nothing) divides 3 damage
    // as 1 and 2 among two Hill Giants: they're dealt 2 and 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obosh, the Preypiercer");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    let giga = t.enter(P0, "Ravenous Gigantotherium");
    t.resolve_all();
    assert_eq!(t.pt(giga), (3, 3), "devoured nothing");
    assert_eq!(dmg(&t, a), 2, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Hill Giant"), "b was dealt 4");
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(a));
}

#[test]
fn the_affected_player_orders_oboshs_doubling_against_a_prevention_shield() {
    cr!("616.1", "616.1e", "615.7");
    ruling!(
        "Obosh, the Preypiercer",
        "If multiple replacement or prevention effects try to modify damage that would be dealt to a permanent or player, the player or the controller of the permanent chooses the order in which they apply."
    );
    supported("Obosh, the Preypiercer");
    supported("Healing Salve");
    // P1 has a shield preventing the next 3 damage dealt to them; P0's Shock (1) would
    // deal 2. Doubling first: 4, 3 prevented, P1 loses 1. Preventing first: all 2
    // prevented, and the shield has 1 left to use on nothing.
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Obosh, the Preypiercer");
        t.set_step(P1, Step::PrecombatMain);
        let salve = t.hand(P1, "Healing Salve");
        mana(&mut t, P1, "W");
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
        t.cast(P1, salve).target(pl(P1)).go();
        t.resolve_all();
        t.set_step(P0, Step::PrecombatMain);
        let from = t.asked().len();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        let shock = t.hand(P0, "Shock");
        mana(&mut t, P0, "R");
        t.cast(P0, shock).target(pl(P1)).go();
        t.resolve_all();
        let options: Vec<Vec<String>> = t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if *p == P1 => Some(options.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(options.len(), 1, "P1 orders the effects: {options:?}");
        assert_eq!(options[0].len(), 2, "{options:?}");
        let doubling_first = options[0][pick].to_lowercase().contains("obosh");
        assert_eq!(t.life(P1), if doubling_first { 19 } else { 20 }, "{options:?}");
    }
}

#[test]
fn bite_down_on_crime_deals_no_damage_when_either_target_is_illegal() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Bite Down on Crime",
        "If either target is an illegal target as Bite Down on Crime tries to resolve, the creature you control won't deal damage."
    );
    supported("Bite Down on Crime");
    // P0's Grizzly Bears and P1's Hill Giant: with the Giant gone, the Bears gets +2/+0
    // and deals nothing; with the Bears gone, nothing happens at all.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    cast(&mut t, P0, "Bite Down on Crime", "3G", &[obj(bears), obj(giant)]);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(dmg(&t, bears), 0);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    let spell = cast(&mut t, P0, "Bite Down on Crime", "3G", &[obj(bears), obj(giant)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(resolved(&t, spell), "one target is still legal (CR 608.2b)");
    assert_eq!(dmg(&t, giant), 0);
    assert!(t.on_battlefield(giant));
    // Both legal: 4 damage to the Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    cast_resolve(&mut t, P0, "Bite Down on Crime", "3G", &[obj(bears), obj(giant)]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}
