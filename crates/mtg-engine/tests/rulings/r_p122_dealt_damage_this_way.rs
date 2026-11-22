//! Rulings batch P122 — "creatures dealt damage this way can't block this turn": Huatli,
//! Warrior Poet and Ballista Watcher // Ballista Wielder, which compile through the phrase
//! added for this batch (every ability of each is exercised).

use crate::r_p122_common::*;
use crate::r_s01_common::{creatures, supported};
use crate::r_s02_common::destroy;
use crate::r_s17_common::enter_transformed;
use crate::r_s29_common::divide;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn can_block(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.can_block_at_all(id)
}

/// Huatli's −X with X = `x`, dividing `amounts` among `targets`.
fn minus_x(t: &mut TestGame, huatli: ObjectId, x: i64, targets: &[ObjectId], amounts: &[i64]) {
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer_targets(P0, &targets.iter().map(|o| obj(*o)).collect::<Vec<_>>());
    divide(t, P0, amounts);
    let uid = crate::r_s27_common::ability_containing(t, huatli, "divided");
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, huatli, uid).unwrap();
    t.g.flush_events();
}

#[test]
fn huatli_minus_x_divides_damage_and_damaged_creatures_cant_block() {
    cr!("601.2d", "606.4", "608.2c");
    supported("Huatli, Warrior Poet");
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let lions = t.battlefield(P1, "Savannah Lions");
    minus_x(&mut t, huatli, 3, &[bears, wurm], &[1, 2]);
    assert_eq!(t.counters(huatli, "loyalty"), 0);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.obj_now(wurm).damage, 2);
    assert!(!can_block(&mut t, bears) && !can_block(&mut t, wurm));
    assert!(can_block(&mut t, lions), "not dealt damage");
}

#[test]
fn huatli_some_targets_illegal_keep_the_division() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Huatli, Warrior Poet",
        "If some (but not all) of the targets become illegal, the original division of damage still applies, but no damage is dealt to illegal targets. If all targets become illegal, the ability won't resolve."
    );
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    t.g.add_counters(Entity::Object(huatli), "loyalty", 2, None);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    minus_x(&mut t, huatli, 3, &[bears, wurm], &[1, 2]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2, "not the bears' 1 damage too");
    assert!(!can_block(&mut t, wurm));
    // All targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    let bears = t.battlefield(P1, "Grizzly Bears");
    minus_x(&mut t, huatli, 2, &[bears], &[2]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(t.g.turn_events.iter().all(|e| !matches!(e, mtg_engine::events::Event::Damage { .. })));
}

#[test]
fn huatli_plus_two_greatest_power_on_resolution() {
    cr!("606.4", "608.2h");
    ruling!(
        "Huatli, Warrior Poet",
        "The greatest power among creatures you control is determined as Huatli's first ability resolves. If that number is negative, you won't gain or lose any life."
    );
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::activate_containing(&mut t, P0, huatli, "gain life").unwrap();
    // A bigger creature arrives before it resolves.
    t.battlefield(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.counters(huatli, "loyalty"), 5);
    // Negative greatest power: no life gained or lost.
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::Const(-4), Value::Const(0))],
    );
    assert_eq!(t.pt(bears).0, -2);
    crate::r_s06_common::activate_containing(&mut t, P0, huatli, "gain life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn huatli_zero_makes_a_dinosaur() {
    cr!("606.4");
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    crate::r_s06_common::activate_containing(&mut t, P0, huatli, "Dinosaur").unwrap();
    t.resolve_all();
    let dinos = crate::r_s01_common::with_subtype(&t, P0, "Dinosaur");
    assert_eq!(dinos.len(), 1);
    assert_eq!(t.pt(dinos[0]), (3, 3));
    assert!(t.obj_now(dinos[0]).has_keyword(KeywordKind::Trample));
    assert_eq!(creatures(&t, P0).len(), 1);
    assert_eq!(t.counters(huatli, "loyalty"), 3);
}

#[test]
fn ballista_wielder_creature_dealt_damage_cant_block() {
    cr!("602.2b", "608.2c");
    supported("Ballista Watcher // Ballista Wielder");
    let mut t = TestGame::new(2);
    let wielder = enter_transformed(&mut t, P0, "Ballista Watcher // Ballista Wielder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 3);
    t.activate(P0, wielder, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(!can_block(&mut t, bears));
    // Its front face's ability taps and has no "can't block" part.
    let mut t = TestGame::new(2);
    let watcher = t.battlefield(P0, "Ballista Watcher // Ballista Wielder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 3);
    t.activate(P0, watcher, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(t.obj_now(watcher).tapped);
    assert!(can_block(&mut t, bears));
    // The back face can target a player.
    let mut t = TestGame::new(2);
    let wielder = enter_transformed(&mut t, P0, "Ballista Watcher // Ballista Wielder");
    mana(&mut t, P0, ManaType::R, 3);
    t.activate(P0, wielder, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}
