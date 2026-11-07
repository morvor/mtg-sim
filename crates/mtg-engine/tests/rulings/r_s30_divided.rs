//! Rulings batch S30 — damage divided as a spell is cast (CR 601.2d, 115.7f, 608.2b):
//! each target is assigned at least 1 damage, the division is locked in (copies keep it),
//! and damage for targets that became illegal isn't dealt. Also excess damage from a
//! spell (CR 120.10).

use crate::r_s01_common::supported;
use crate::r_s25_common::{cast_new, change_copy_targets, spell_copies, targets_of};
use crate::r_s28_common::cast_card;
use crate::r_s29_common::damage_marked;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The division of a spell on the stack among the targets of its first slot.
fn division(t: &TestGame, id: ObjectId) -> Vec<u32> {
    t.g.obj(id)
        .stack
        .as_deref()
        .and_then(|si| si.chosen.first())
        .and_then(|c| c.divided.first().cloned())
        .unwrap_or_default()
}

/// P0 casts Arc Lightning ("deals 3 damage divided as you choose among one, two, or three
/// targets"): 2 to `bears`, 1 to P1.
fn arc(t: &mut TestGame, bears: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let arc = cast_card(t, P0, "Arc Lightning");
    assert_eq!(division(t, arc), vec![2, 1]);
    arc
}

/// Checks the one copy of the Arc on the stack (retargeted from the Bears to the Giant)
/// and resolves everything.
fn check_copy(t: &mut TestGame, arc: ObjectId, giant: ObjectId) {
    let copies = spell_copies(t);
    assert_eq!(copies.len(), 1);
    assert_eq!(
        targets_of(t, copies[0]),
        vec![Entity::Object(giant), Entity::Player(P1)]
    );
    assert_eq!(division(t, copies[0]), vec![2, 1]);
    assert_eq!(division(t, arc), vec![2, 1]);
    t.resolve_all();
    assert_eq!(damage_marked(t, giant), 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn each_target_of_pyrokinesis_must_be_assigned_at_least_1_damage() {
    cr!("601.2d");
    ruling!("Pyrokinesis", "Each target must be assigned at least 1 damage.");
    supported("Pyrokinesis");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    // An attempt to assign all 4 to one target and none to the other isn't allowed.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![4, 0]));
    let from = t.asked().len();
    let spell = cast_card(&mut t, P0, "Pyrokinesis");
    assert!(t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::Divide {
            total: 4,
            min_each: 1,
            ..
        }
    )));
    let div = division(&t, spell);
    assert_eq!(div.iter().sum::<u32>(), 4);
    assert!(div.iter().all(|n| *n >= 1));
    t.resolve_all();
    assert!(damage_marked(&t, a) >= 1 && damage_marked(&t, b) >= 1);
}

#[test]
fn swarm_intelligences_copy_keeps_the_division() {
    cr!("707.10c", "115.7f");
    ruling!(
        "Swarm Intelligence",
        "If the spell has damage divided as it was cast (like Chandra’s Pyrohelix), the division can’t be changed (although the targets receiving that damage still can)."
    );
    supported("Swarm Intelligence");
    let mut t = TestGame::new(2);
    // "Whenever you cast an instant or sorcery spell, you may copy that spell. You may
    // choose new targets for the copy."
    t.battlefield(P0, "Swarm Intelligence");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let arc = arc(&mut t, bears);
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant)), None]);
    t.resolve();
    check_copy(&mut t, arc, giant);
}

#[test]
fn insidious_wills_copy_keeps_the_division() {
    cr!("707.10c", "115.7f");
    ruling!(
        "Insidious Will",
        "If the spell has damage divided as it was cast (like Chandra's Pyrohelix), the division can't be changed (although the targets receiving that damage still can)."
    );
    supported("Insidious Will");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let arc = arc(&mut t, bears);
    // "Copy target instant or sorcery spell. You may choose new targets for the copy."
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    cast_new(&mut t, P0, "Insidious Will", &[Entity::Object(arc)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant)), None]);
    t.resolve();
    check_copy(&mut t, arc, giant);
}

#[test]
fn damage_divided_to_a_target_that_became_illegal_isnt_dealt() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Nahiri's Sacrifice",
        "If some of the targets of the last ability become illegal, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn't dealt at all."
    );
    supported("Nahiri's Sacrifice");
    let mut t = TestGame::new(2);
    // "As an additional cost to cast this spell, sacrifice an artifact or creature with
    // mana value X. Nahiri's Sacrifice deals X damage divided as you choose among any
    // number of target creatures." Sacrificing Hill Giant: X is 4.
    let fodder = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(wurm)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 2]));
    let spell = cast_card(&mut t, P0, "Nahiri's Sacrifice");
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(division(&t, spell), vec![2, 2]);
    // In response, the Bears leave: the 2 damage for them isn't dealt to anything, and the
    // Craw Wurm is still dealt only its 2.
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(damage_marked(&t, wurm), 2);
    assert!(t.on_battlefield(wurm));
}

#[test]
fn excess_damage_from_a_spell_ignores_prevention_and_indestructible() {
    cr!("120.4a", "120.4b", "120.6");
    ruling!(
        "Flame Spill",
        "Excess damage caused by a spell or ability is similar to how combat damage from a creature with trample is handled. Start with the amount of damage being dealt to the creature and determine what is “lethal.”"
    );
    supported("Flame Spill");
    // "Flame Spill deals 4 damage to target creature. Excess damage is dealt to that
    // creature's controller instead."
    // Damage already marked counts: Hill Giant with 2 damage has 1 lethal damage left.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Shock", &[Entity::Object(giant)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Flame Spill", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Indestructible doesn't matter: Darksteel Myr (0/1) is dealt 1, its controller 3.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_new(&mut t, P0, "Flame Spill", &[Entity::Object(myr)]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(damage_marked(&t, myr), 1);
    assert_eq!(t.life(P1), 17);
    // Prevention doesn't matter either: Grizzly Bears with a Healing Salve shield ("Prevent
    // the next 3 damage that would be dealt to any target this turn"): 2 is assigned to
    // it (and prevented), and the other 2 is dealt to its controller.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P1, "Healing Salve", &[Entity::Object(bears)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Flame Spill", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, bears), 0);
    assert_eq!(t.life(P1), 18);
}
