//! Rulings batch P007 — damage divided among targets: the division is fixed as the spell
//! is cast or the ability is put on the stack (CR 601.2d, 603.3d), each target gets at
//! least 1, and damage for targets that became illegal isn't dealt (CR 608.2b).

use crate::r_p007_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s07_common::resolved;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// The division of the top object of the stack (all slots).
fn division(t: &TestGame) -> Vec<u32> {
    let top = *t.g.stack.last().expect("stack is empty");
    t.g.obj(top)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.divided.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// How a divided-damage spell or ability is put on the stack, targeting `targets` with
/// the queued division.
#[derive(Clone, Copy, Debug)]
enum Divider {
    /// A spell with X (or none).
    Spell(&'static str, Option<i64>),
    /// A creature's enters trigger.
    Enters(&'static str),
    /// A planeswalker's loyalty ability (name, index, loyalty).
    Walker(&'static str, usize, u32),
}

fn start(t: &mut TestGame, d: Divider, targets: &[Entity], split: &[i64]) -> Option<ObjectId> {
    divide(t, P0, split);
    match d {
        Divider::Spell(name, x) => {
            t.answer_targets(P0, targets);
            supported(name);
            Some(cast_x(t, P0, name, NORMAL, x, &[]))
        }
        Divider::Enters(name) => {
            supported(name);
            t.answer_targets(P0, targets);
            t.enter(P0, name);
            t.settle();
            None
        }
        Divider::Walker(name, i, l) => {
            supported(name);
            let w = pw(t, P0, name, l);
            t.answer_targets(P0, targets);
            loyalty(t, P0, w, i, &[]);
            None
        }
    }
}

#[test]
fn division_stays_when_some_targets_become_illegal() {
    cr!("608.2b", "601.2d", "603.3d");
    ruling!(
        "Fury",
        "If some of the targets are illegal as the triggered ability tries to resolve, the original division of damage still applies, but the damage that would have been dealt to the illegal targets isn't dealt at all."
    );
    ruling!(
        "Dragonlord Atarka",
        "If some of the targets are illegal targets as the ability tries to resolve, the original division of damage still applies but no damage is dealt to the illegal targets."
    );
    ruling!(
        "Mythos of Vadrok",
        "If some of the targets become illegal for Mythos of Vadrok, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn’t dealt at all."
    );
    ruling!(
        "Chandra, Flameshaper",
        "If some of the targets of Chandra's last ability become illegal, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn't dealt at all."
    );
    ruling!(
        "Meteor Swarm",
        "If Meteor Swarm has one or more targets, and some (but not all) of them are illegal as Meteor Swarm tries to resolve, the amount of damage dealt to the remaining legal targets doesn't change."
    );
    ruling!(
        "Shatterskull Smashing // Shatterskull, the Hammer Pass",
        "If one of the targets becomes illegal for Shatterskull Smashing, the original division of damage still applies to the still-legal target, but the damage that would have been dealt to the illegal target isn't dealt at all."
    );
    // (divider, total damage)
    let cases = [
        (Divider::Enters("Fury"), 4),
        (Divider::Enters("Dragonlord Atarka"), 5),
        (Divider::Spell("Mythos of Vadrok", None), 5),
        (Divider::Walker("Chandra, Flameshaper", 2, 6), 8),
        (Divider::Spell("Meteor Swarm", Some(2)), 8),
        (
            Divider::Spell(
                "Shatterskull Smashing // Shatterskull, the Hammer Pass",
                Some(4),
            ),
            4,
        ),
    ];
    for (d, total) in cases {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Colossal Dreadmaw");
        let b = t.battlefield(P1, "Colossal Dreadmaw");
        let first = total as i64 - 1;
        start(&mut t, d, &[obj(a), obj(b)], &[first, 1]);
        assert_eq!(division(&t), vec![first as u32, 1], "{d:?}");
        destroy(&mut t, a);
        t.resolve_all();
        assert_eq!(dmg(&t, b), 1, "{d:?}: b still gets only its share");
    }
}

#[test]
fn with_all_targets_illegal_the_divided_damage_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Dragonlord Atarka",
        "If all targets are illegal, the ability doesn’t resolve."
    );
    ruling!(
        "Meteor Swarm",
        "If all targets become illegal, Meteor Swarm won't resolve."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    let s = start(
        &mut t,
        Divider::Spell("Meteor Swarm", Some(2)),
        &[obj(a), obj(b)],
        &[4, 4],
    )
    .unwrap();
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert!(t.in_graveyard(P0, "Meteor Swarm"));
    // Atarka's trigger is removed from the stack without effect.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    start(
        &mut t,
        Divider::Enters("Dragonlord Atarka"),
        &[obj(a), obj(b)],
        &[3, 2],
    );
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn each_target_of_a_division_gets_at_least_one() {
    cr!("601.2d");
    ruling!(
        "Mythos of Vadrok",
        "Each chosen target must receive at least 1 damage."
    );
    ruling!(
        "Sundering Stroke",
        "Each chosen target must receive at least 1 damage."
    );
    for (name, total) in [("Mythos of Vadrok", 5u32), ("Sundering Stroke", 7)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Colossal Dreadmaw");
        let b = t.battlefield(P1, "Colossal Dreadmaw");
        start(
            &mut t,
            Divider::Spell(name, None),
            &[obj(a), obj(b)],
            &[total as i64, 0],
        );
        let d = division(&t);
        assert_eq!(d.iter().sum::<u32>(), total, "{name}");
        assert_eq!(d.len(), 2, "{name}");
        assert!(d.iter().all(|n| *n >= 1), "{name}: {d:?}");
    }
}

#[test]
fn shatterskull_divides_twice_x() {
    cr!("601.2d", "107.3a");
    ruling!(
        "Shatterskull Smashing // Shatterskull, the Hammer Pass",
        "If X is 6 or more, you divide twice X damage; you don't assign X damage and then double that damage."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    let d = Divider::Spell(
        "Shatterskull Smashing // Shatterskull, the Hammer Pass",
        Some(6),
    );
    start(&mut t, d, &[obj(a), obj(b)], &[7, 5]);
    assert_eq!(division(&t), vec![7, 5]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(dmg(&t, b), 5);
}

#[test]
fn lukka_x_is_locked_in_as_the_ability_is_activated() {
    cr!("601.2d", "602.2b", "608.2h");
    ruling!(
        "Lukka, Bound to Ruin",
        "Once the ability has been activated, the greatest power among creatures you control no longer matters for this ability."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    start(
        &mut t,
        Divider::Walker("Lukka, Bound to Ruin", 2, 5),
        &[obj(a), obj(b)],
        &[4, 2],
    );
    assert_eq!(division(&t), vec![4, 2]);
    // The Craw Wurm is destroyed in response: the damage doesn't change.
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (4, 2));
}
