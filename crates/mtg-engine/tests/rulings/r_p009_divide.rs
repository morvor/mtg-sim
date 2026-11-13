//! Rulings batch P009 — damage and counters divided among targets: the division is chosen
//! as the spell is cast or the ability is put on the stack (CR 601.2d, 603.3d), each target
//! gets at least 1 (CR 601.2d), and it doesn't change if a target becomes illegal
//! (CR 608.2b). Also Mythos of Vadrok's restrictions on its targets.

use crate::r_p009_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::targets_of;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn divide(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}

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

#[test]
fn division_is_chosen_as_the_spell_is_cast() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Meteor Swarm",
        "You divide the damage as you cast Meteor Swarm. Each target must be assigned at least 1 damage. Because there is a maximum of eight targets, X can't be greater than 8."
    );
    ruling!(
        "Shatterskull Smashing // Shatterskull, the Hammer Pass",
        "You divide the damage as you cast Shatterskull Smashing, not as it resolves. If you choose two targets, each target must be assigned at least 1 damage."
    );
    supported("Meteor Swarm");
    supported("Shatterskull Smashing // Shatterskull, the Hammer Pass");
    // Meteor Swarm, X = 2: 5 and 3, fixed on the stack.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 5);
    let swarm = t.hand(P0, "Meteor Swarm");
    divide(&mut t, P0, &[5, 3]);
    t.cast(P0, swarm).x(2).targets(&[obj(a), obj(b)]).go();
    assert_eq!(division(&t), vec![5, 3]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(dmg(&t, b), 3);
    // A division giving a target 0 isn't allowed: each gets at least 1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 5);
    let swarm = t.hand(P0, "Meteor Swarm");
    divide(&mut t, P0, &[8, 0]);
    t.cast(P0, swarm).x(2).targets(&[obj(a), obj(b)]).go();
    let d = division(&t);
    assert_eq!(d.iter().sum::<u32>(), 8);
    assert!(d.iter().all(|n| *n >= 1), "{d:?}");
    // X can't be 9: there's no way to give nine targets at least 1 of 8 damage.
    let mut t = TestGame::new(2);
    let wurms: Vec<Entity> = (0..9)
        .map(|_| obj(t.battlefield(P1, "Craw Wurm")))
        .collect();
    t.lands(P0, "Mountain", 12);
    let swarm = t.hand(P0, "Meteor Swarm");
    let r = t.cast(P0, swarm).x(9).targets(&wurms).try_go();
    t.clear_answers();
    if let Ok(s) = r {
        assert!(targets_of(&t, s).len() <= 8);
    }

    // Shatterskull Smashing, X = 3 among two targets: 2 and 1, fixed as it's cast.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 5);
    let ss = t.hand(P0, "Shatterskull Smashing // Shatterskull, the Hammer Pass");
    divide(&mut t, P0, &[2, 1]);
    t.cast(P0, ss).x(3).targets(&[obj(a), obj(b)]).go();
    assert_eq!(division(&t), vec![2, 1]);
    kill(&mut t, a);
    t.resolve_all();
    assert_eq!(dmg(&t, b), 1, "the division doesn't change");
}

#[test]
fn division_is_chosen_as_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "601.2d");
    ruling!(
        "Fury",
        "You divide the damage as you put the triggered ability on the stack, not as it resolves. Each target must be assigned at least 1 damage. You can't choose more than four targets and deal 0 damage to some of them."
    );
    ruling!(
        "Dragonlord Atarka",
        "You choose how many targets the ability has and how the damage is divided as you put the ability on the stack."
    );
    supported("Fury");
    supported("Dragonlord Atarka");
    // Fury: five targets is too many for 4 damage.
    let mut t = TestGame::new(2);
    let bears: Vec<Entity> = (0..5)
        .map(|_| obj(t.battlefield(P1, "Grizzly Bears")))
        .collect();
    t.answer_targets(P0, &bears);
    t.enter(P0, "Fury");
    t.settle();
    t.clear_answers();
    // At most four targets are offered; the five-target answer is rejected.
    assert!(t.asked().iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { max: 4, .. }
    )));
    assert_eq!(t.stack_len(), 1, "the trigger is put on the stack");
    let top = t.g.stack[0];
    assert!(targets_of(&t, top).len() <= 4);
    assert!(division(&t).iter().all(|n| *n >= 1));
    // Fury: 3 and 1 among two targets, fixed on the stack.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[3, 1]);
    t.enter(P0, "Fury");
    t.settle();
    assert_eq!(division(&t), vec![3, 1]);
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (3, 1));
    // Dragonlord Atarka: three targets, 3/1/1.
    let mut t = TestGame::new(2);
    let ws: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
    t.answer_targets(P0, &ws.iter().map(|w| obj(*w)).collect::<Vec<_>>());
    divide(&mut t, P0, &[3, 1, 1]);
    t.enter(P0, "Dragonlord Atarka");
    t.settle();
    assert_eq!(division(&t), vec![3, 1, 1]);
    t.resolve_all();
    assert_eq!(
        ws.iter().map(|w| dmg(&t, *w)).collect::<Vec<_>>(),
        vec![3, 1, 1]
    );
}

#[test]
fn ureni_fixes_x_and_the_division_when_it_triggers() {
    cr!("603.3d", "601.2d", "608.2b");
    ruling!(
        "Ureni, the Song Unending",
        "You cannot choose more than X targets for Ureni’s triggered ability. You choose how the damage is divided immediately after choosing those targets. The value of X won’t change even if the number of lands you control changes after that point. That division of damage also won’t change, even if damage can’t be dealt to one or more of those targets as the ability resolves (usually because they’ve left the battlefield)."
    );
    supported("Ureni, the Song Unending");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 3);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[2, 1]);
    t.enter(P0, "Ureni, the Song Unending");
    t.settle();
    assert_eq!(division(&t), vec![2, 1]);
    // More lands, then fewer, and one target leaves: still 2 to the other.
    add(&mut t, P0, "Forest");
    kill(&mut t, lands[0]);
    kill(&mut t, lands[1]);
    kill(&mut t, b);
    t.resolve_all();
    assert_eq!(dmg(&t, a), 2);
    // No more than X targets: with one land, only one target.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.enter(P0, "Ureni, the Song Unending");
    t.settle();
    t.clear_answers();
    // X = 1: at most one target is offered; the two-target answer is rejected.
    assert!(t.asked().iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { max: 1, .. }
    )));
    assert_eq!(t.stack_len(), 1, "the trigger is put on the stack");
    let top = t.g.stack[0];
    assert!(targets_of(&t, top).len() <= 1);
}

#[test]
fn lukka_limits_targets_to_the_greatest_power() {
    cr!("601.2d", "602.2b");
    ruling!(
        "Lukka, Bound to Ruin",
        "You can't choose more targets than the greatest power among creatures you control as you activate the ability, and each chosen target must receive at least 1 damage."
    );
    supported("Lukka, Bound to Ruin");
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Bound to Ruin");
    t.battlefield(P0, "Grizzly Bears");
    let ws: Vec<Entity> = (0..3)
        .map(|_| obj(t.battlefield(P1, "Craw Wurm")))
        .collect();
    t.answer_targets(P0, &ws);
    divide(&mut t, P0, &[1, 1, 0]);
    let r = activate_containing(&mut t, P0, lukka, "divided");
    t.clear_answers();
    if let Ok(Some(ab)) = r {
        assert!(targets_of(&t, ab).len() <= 2);
        assert!(division(&t).iter().all(|n| *n >= 1));
    } else {
        assert_eq!(t.stack_len(), 0, "{r:?}");
    }
    // Greatest power 2 (Grizzly Bears): at most two targets.
    assert!(t.asked().iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { max: 2, .. }
    )));
    // Two targets with power 2: 1 each.
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Bound to Ruin");
    t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[1, 1]);
    activate_containing(&mut t, P0, lukka, "divided").unwrap();
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (1, 1));
}

#[test]
fn vivien_distributes_counters_as_she_activates() {
    cr!("601.2d", "115.1d");
    ruling!(
        "Vivien, Arkbow Ranger",
        "You can activate Vivien’s first ability without choosing any target creatures. The counters won’t be put on anything."
    );
    ruling!(
        "Vivien, Arkbow Ranger",
        "You choose how the counters will be distributed as you activate Vivien’s first ability. Each target creature must be assigned at least one counter. This means that you can’t put two counters on one creature but give two creatures trample."
    );
    supported("Vivien, Arkbow Ranger");
    // No targets: it still resolves (and her loyalty goes up).
    let mut t = TestGame::new(2);
    let vivien = t.battlefield(P0, "Vivien, Arkbow Ranger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.activate(P0, vivien, 0, &[]).unwrap();
    t.clear_answers();
    t.resolve_all();
    assert_eq!(t.counters(vivien, counters::LOYALTY), 5);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // Two targets: one counter each; a 2/0 split isn't allowed.
    let mut t = TestGame::new(2);
    let vivien = t.battlefield(P0, "Vivien, Arkbow Ranger");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[2, 0]);
    t.activate(P0, vivien, 0, &[]).unwrap();
    assert_eq!(division(&t), vec![1, 1]);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn mythos_of_vadrok_restricts_targets_even_if_damage_is_prevented() {
    cr!("608.2b", "615.1");
    ruling!(
        "Mythos of Vadrok",
        "The target creatures and planeswalkers will be unable to attack, block, or have their abilities activated even if the damage that would be dealt to them is prevented. However, if one of those permanents becomes an illegal target, it will be able to attack, block, and have its abilities activated."
    );
    supported("Mythos of Vadrok");
    supported("Inviolability");
    supported("Blossoming Defense");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    crate::r_s06_common::attach_new(&mut t, P1, "Inviolability", obj(sorcerer));
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let mythos = t.hand(P0, "Mythos of Vadrok");
    divide(&mut t, P0, &[1, 4]);
    t.cast(P0, mythos).targets(&[obj(sorcerer), obj(wurm)]).go();
    // The Wurm becomes an illegal target (hexproof) in response.
    t.lands(P1, "Forest", 1);
    let bd = t.hand(P1, "Blossoming Defense");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bd).target(wurm).go();
    t.resolve(); // Blossoming Defense
    t.resolve_all();
    assert_eq!(dmg(&t, sorcerer), 0, "prevented");
    assert!(t.on_battlefield(sorcerer));
    assert!(!t.g.can_attack(sorcerer));
    assert!(!t.g.can_block_at_all(sorcerer));
    assert!(t.activate(P1, sorcerer, 0, &[pl(P0)]).is_err());
    // The Wurm was an illegal target: no damage, no restrictions.
    assert_eq!(dmg(&t, wurm), 0);
    assert!(t.g.can_attack(wurm));
    assert!(t.g.can_block_at_all(wurm));
}
