//! Rulings batch P109 — damage divided among targets: the division is announced as the
//! spell is cast or the triggered ability is put on the stack, each target gets at least 1
//! (CR 601.2d, 603.3d); it doesn't change when some targets become illegal, and the spell
//! doesn't resolve when all of them are (CR 608.2b); copies keep it (CR 707.10).

use crate::r_p009_common::{dmg, kill, obj, pl};
use crate::r_p030_common::resolve_until_copies;
use crate::r_p109_common::*;
use crate::r_s01_common::supported;
use crate::r_s25_common::{cast_new, change_copy_targets, targets_of};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// One "divided as you choose" spell: its name and X (if it has {X}).
struct Divided {
    name: &'static str,
    x: Option<i64>,
}

const PYROHELIX: Divided = Divided {
    name: "Chandra's Pyrohelix",
    x: None,
};
const ELECTROLYZE: Divided = Divided {
    name: "Electrolyze",
    x: None,
};
const FORKED_BOLT: Divided = Divided {
    name: "Forked Bolt",
    x: None,
};
const TWIN_BOLT: Divided = Divided {
    name: "Twin Bolt",
    x: None,
};
const BOULDERFALL: Divided = Divided {
    name: "Boulderfall",
    x: None,
};
const PYROTECHNICS: Divided = Divided {
    name: "Pyrotechnics",
    x: None,
};
const SPREADING_FLAMES: Divided = Divided {
    name: "Spreading Flames",
    x: None,
};
const ROLLING_THUNDER: Divided = Divided {
    name: "Rolling Thunder",
    x: Some(4),
};

/// The total damage of the spell.
fn total(d: &Divided) -> i64 {
    match d.name {
        "Boulderfall" => 5,
        "Pyrotechnics" => 4,
        "Spreading Flames" => 6,
        "Rolling Thunder" => d.x.unwrap(),
        _ => 2,
    }
}

/// The spell is cast with the division `amounts` among that many Craw Wurms (6/4): the
/// division is fixed on the stack as cast; an answer giving a target 0 is rejected.
fn division_is_announced_as_cast(d: &Divided, amounts: &[i64]) {
    supported(d.name);
    let mut t = TestGame::new(2);
    let ws: Vec<ObjectId> = amounts
        .iter()
        .map(|_| t.battlefield(P1, "Craw Wurm"))
        .collect();
    let es: Vec<Entity> = ws.iter().map(|w| obj(*w)).collect();
    let s = cast_divided(&mut t, d.name, d.x, &es, amounts);
    assert!(
        t.asked()
            .iter()
            .any(|(_, q)| matches!(q, Decision::Divide { .. })),
        "{}: divided as it's cast",
        d.name
    );
    let want: Vec<u32> = amounts.iter().map(|n| *n as u32).collect();
    assert_eq!(division_of(&t, s), want, "{}", d.name);
    t.resolve_all();
    let got: Vec<u32> = ws.iter().map(|w| dmg(&t, *w)).collect();
    assert_eq!(got, want, "{}", d.name);
    // Each target must be assigned at least 1: a division with a 0 isn't accepted.
    if amounts.len() >= 2 {
        let mut t = TestGame::new(2);
        let es: Vec<Entity> = amounts
            .iter()
            .map(|_| obj(t.battlefield(P1, "Craw Wurm")))
            .collect();
        let mut bad = vec![0; amounts.len()];
        bad[amounts.len() - 1] = total(d);
        let s = cast_divided(&mut t, d.name, d.x, &es, &bad);
        let div = division_of(&t, s);
        assert_eq!(div.iter().sum::<u32>() as i64, total(d), "{}", d.name);
        assert!(div.iter().all(|n| *n >= 1), "{}: {div:?}", d.name);
    }
}

#[test]
fn the_division_is_announced_as_the_spell_is_cast() {
    cr!("601.2d");
    ruling!(
        "Chandra's Pyrohelix",
        "You divide the damage as you cast Chandra's Pyrohelix, not as it resolves. Each target must be assigned at least 1 damage."
    );
    division_is_announced_as_cast(&PYROHELIX, &[1, 1]);
    ruling!(
        "Electrolyze",
        "You divide the damage as you cast Electrolyze, not as it resolves. Each target must be assigned at least 1 damage."
    );
    division_is_announced_as_cast(&ELECTROLYZE, &[1, 1]);
    ruling!(
        "Forked Bolt",
        "You divide the damage as you cast Forked Bolt, not as it resolves. Each target must be assigned at least 1 damage."
    );
    division_is_announced_as_cast(&FORKED_BOLT, &[1, 1]);
    ruling!(
        "Boulderfall",
        "You divide the damage as you cast the spell, not as it resolves. Each target must be assigned at least 1 damage."
    );
    division_is_announced_as_cast(&BOULDERFALL, &[3, 1, 1]);
    ruling!(
        "Pyrotechnics",
        "You choose how the damage is divided as you cast the spell, not as it resolves. Each target must be assigned at least 1 damage."
    );
    division_is_announced_as_cast(&PYROTECHNICS, &[1, 1, 1, 1]);
    ruling!(
        "Spreading Flames",
        "You announce how the damage will be divided as part of casting Spreading Flames. Each chosen target must receive at least 1 damage."
    );
    division_is_announced_as_cast(&SPREADING_FLAMES, &[3, 2, 1]);
    ruling!(
        "Rolling Thunder",
        "You choose how many targets Rolling Thunder has and how the damage is divided as you cast the spell. Each target must receive at least 1 damage."
    );
    division_is_announced_as_cast(&ROLLING_THUNDER, &[2, 1, 1]);
}

#[test]
fn two_damage_to_one_target_or_one_to_each_of_two() {
    cr!("601.2d", "601.2c");
    for (d, line) in [
        (&TWIN_BOLT, 0),
        (&PYROHELIX, 1),
        (&ELECTROLYZE, 2),
        (&FORKED_BOLT, 3),
    ] {
        let _ = match line {
            0 => ruling!(
                "Twin Bolt",
                "You choose whether Twin Bolt has one target that will be dealt 2 damage or two targets that will each be dealt 1 damage as you cast Twin Bolt."
            ),
            1 => ruling!(
                "Chandra's Pyrohelix",
                "as you cast Chandra's Pyrohelix, you choose whether to have it deal 2 damage to a single target, or deal 1 damage to each of two targets."
            ),
            2 => ruling!(
                "Electrolyze",
                "as you cast Electrolyze, you choose whether to have it deal 2 damage to a single target, or deal 1 damage to each of two targets."
            ),
            _ => ruling!(
                "Forked Bolt",
                "as you cast Forked Bolt, you choose whether to have it deal 2 damage to a single target, or deal 1 damage to each of two targets."
            ),
        };
        supported(d.name);
        // One target: 2 damage to it.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let s = cast_divided(&mut t, d.name, None, &[obj(bears)], &[]);
        assert_eq!(targets_of(&t, s), vec![obj(bears)], "{}", d.name);
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{}: 2 damage kills it", d.name);
        // Two targets: 1 to each.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_divided(&mut t, d.name, None, &[obj(bears), pl(P1)], &[1, 1]);
        t.resolve_all();
        assert_eq!(dmg(&t, bears), 1, "{}", d.name);
        assert_eq!(t.life(P1), 19, "{}", d.name);
        // There's no third target.
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        let r = try_cast_divided(&mut t, d.name, None, &[obj(a), obj(b), pl(P1)], &[1, 1]);
        if let Ok(s) = r {
            assert!(targets_of(&t, s).len() <= 2, "{}", d.name);
        }
    }
}

/// Cast with 1 to a Grizzly Bears and 1 to a Hill Giant (3/3); the Bears leaves before it
/// resolves: the Giant is dealt 1 damage, not 2.
fn one_target_left_two_damage_spell(d: &Divided) -> TestGame {
    supported(d.name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let s = cast_divided(&mut t, d.name, None, &[obj(bears), obj(giant)], &[1, 1]);
    assert_eq!(division_of(&t, s), vec![1, 1]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(dmg(&t, giant), 1, "{}: not 2", d.name);
    t
}

#[test]
fn the_remaining_target_is_dealt_one_damage_not_two() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Chandra's Pyrohelix",
        "If Chandra's Pyrohelix targets two creatures and one becomes an illegal target, the remaining target is dealt 1 damage, not 2."
    );
    one_target_left_two_damage_spell(&PYROHELIX);
    ruling!(
        "Twin Bolt",
        "If you cast Twin Bolt with two targets and one of those targets becomes illegal before Twin Bolt resolves, the remaining legal target will be dealt 1 damage. You can't change the original division of damage."
    );
    ruling!(
        "Twin Bolt",
        "If you cast Twin Bolt with two targets and one of those targets becomes illegal before Twin Bolt resolves, the remaining legal target will be dealt 1 damage. You can’t change the original division of damage."
    );
    one_target_left_two_damage_spell(&TWIN_BOLT);
    ruling!(
        "Electrolyze",
        "If Electrolyze targets two creatures and one becomes an illegal target, the remaining target is dealt 1 damage, not 2. You'll still draw a card."
    );
    let hand = 0;
    let t = one_target_left_two_damage_spell(&ELECTROLYZE);
    assert_eq!(t.hand_size(P0), hand + 1, "you still draw a card");
}

#[test]
fn electrolyze_with_all_targets_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Electrolyze",
        "If all chosen targets are illegal as Electrolyze tries to resolve, it won't resolve and none of its effects will happen. No damage will be dealt and you won't draw a card."
    );
    supported("Electrolyze");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_divided(&mut t, "Electrolyze", None, &[obj(bears), obj(giant)], &[1, 1]);
    kill(&mut t, bears);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0, "no card drawn");
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Electrolyze"));
}

/// Cast with 3/2/1 among three Craw Wurms (or 4 X); the middle one leaves: the others are
/// dealt their original shares; with all of them gone the spell doesn't resolve.
fn some_targets_illegal(d: &Divided, amounts: &[i64; 3]) {
    supported(d.name);
    let mut t = TestGame::new(2);
    let ws: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
    let es: Vec<Entity> = ws.iter().map(|w| obj(*w)).collect();
    cast_divided(&mut t, d.name, d.x, &es, amounts);
    kill(&mut t, ws[1]);
    t.resolve_all();
    assert_eq!(dmg(&t, ws[0]), amounts[0] as u32, "{}", d.name);
    assert_eq!(dmg(&t, ws[2]), amounts[2] as u32, "{}", d.name);
    // All targets illegal: it doesn't resolve (no damage anywhere, still goes to the
    // graveyard).
    let mut t = TestGame::new(2);
    let ws: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
    let other = t.battlefield(P1, "Craw Wurm");
    let es: Vec<Entity> = ws.iter().map(|w| obj(*w)).collect();
    cast_divided(&mut t, d.name, d.x, &es, amounts);
    for w in &ws {
        kill(&mut t, *w);
    }
    t.resolve_all();
    assert_eq!(dmg(&t, other), 0, "{}", d.name);
    assert_eq!(t.life(P1), 20, "{}", d.name);
    assert!(t.in_graveyard(P0, d.name), "{}", d.name);
}

#[test]
fn the_original_division_applies_when_some_targets_are_illegal() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Spreading Flames",
        "If Spreading Flames has multiple targets, and some, but not all, are illegal targets when Spreading Flames tries to resolve, Spreading Flames will still deal damage to the remaining legal targets according to the original damage division. If all the targets are illegal, the spell won’t resolve."
    );
    some_targets_illegal(&SPREADING_FLAMES, &[3, 2, 1]);
    ruling!(
        "Rolling Thunder",
        "If some, but not all, of the targets become illegal by the time Rolling Thunder tries to resolve, the original division of damage still applies, but no damage is dealt to illegal targets. If all the targets are illegal, the spell won’t resolve."
    );
    some_targets_illegal(&ROLLING_THUNDER, &[2, 1, 1]);
    ruling!(
        "Boulderfall",
        "If some but not all of Boulderfall's targets become illegal, you can't change the division of damage. Damage that would have been dealt to illegal targets simply isn't dealt."
    );
    some_targets_illegal(&BOULDERFALL, &[2, 2, 1]);
    ruling!(
        "Pyrotechnics",
        "If some but not all of Pyrotechnics’s targets become illegal, you can’t change the division of damage. Damage that would have been dealt to illegal targets simply isn’t dealt."
    );
    some_targets_illegal(&PYROTECHNICS, &[1, 2, 1]);
}

#[test]
fn bogardan_hellkite_divides_as_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "601.2d", "608.2b");
    ruling!(
        "Bogardan Hellkite",
        "You choose how many targets Bogardan Hellkite's triggered ability has and how the damage is divided as you put the ability onto the stack. Each target must receive at least 1 damage."
    );
    ruling!(
        "Bogardan Hellkite",
        "If some of the creatures or players are illegal targets as Bogardan Hellkite's triggered ability tries to resolve, the original division of damage still applies and the damage that would have been dealt to the illegal targets is lost."
    );
    supported("Bogardan Hellkite");
    let mut t = TestGame::new(2);
    let ws: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
    t.answer_targets(P0, &[obj(ws[0]), obj(ws[1]), pl(P1)]);
    divide(&mut t, P0, &[3, 1, 1]);
    t.enter(P0, "Bogardan Hellkite");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    assert_eq!(division_of(&t, trig), vec![3, 1, 1]);
    assert_eq!(targets_of(&t, trig).len(), 3);
    // The second Wurm leaves: its 1 is lost, not dealt to another target.
    kill(&mut t, ws[1]);
    t.resolve_all();
    assert_eq!(dmg(&t, ws[0]), 3);
    assert_eq!(dmg(&t, ws[2]), 0);
    assert_eq!(t.life(P1), 19);
    // Each target must receive at least 1: a 0 isn't accepted.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), pl(P1)]);
    divide(&mut t, P0, &[5, 0]);
    t.enter(P0, "Bogardan Hellkite");
    t.settle();
    let div = division_of(&t, t.g.stack[0]);
    assert_eq!(div.iter().sum::<u32>(), 5);
    assert!(div.iter().all(|n| *n >= 1), "{div:?}");
}

/// P0 casts the spell with 2 to a Craw Wurm and the rest to P1 (two targets), then
/// Twincast copies it, retargeting the Wurm part to another Wurm: the copy keeps the
/// division.
fn copy_keeps_division(d: &Divided) {
    supported(d.name);
    supported("Twincast");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    let rest = total(d) - 2;
    let s = cast_divided(&mut t, d.name, d.x, &[obj(a), pl(P1)], &[2, rest]);
    cast_new(&mut t, P0, "Twincast", &[obj(s)]);
    change_copy_targets(&mut t, P0, &[Some(obj(b)), None]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", d.name);
    assert_eq!(division_of(&t, copies[0]), vec![2, rest as u32], "{}", d.name);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(b), pl(P1)], "{}", d.name);
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (2, 2), "{}", d.name);
    assert_eq!(t.life(P1), 20 - 2 * rest as i32, "{}", d.name);
}

#[test]
fn a_copy_keeps_the_division_and_may_change_targets() {
    cr!("707.10", "707.10c");
    ruling!(
        "Rolling Thunder",
        "If Rolling Thunder is copied, the copy will retain the original damage divsion. The spell or ability will likely allow you to choose new targets, but you can’t change the way damage is divided."
    );
    copy_keeps_division(&ROLLING_THUNDER);
    ruling!(
        "Boulderfall",
        "If an effect creates a copy of Boulderfall, the division of damage and number of targets can't be changed. The effect that creates the copy may allow you to change targets, however."
    );
    copy_keeps_division(&BOULDERFALL);
    ruling!(
        "Pyrotechnics",
        "If an effect creates a copy of Pyrotechnics, the number of targets and division of damage can’t be changed. The effect that creates the copy may allow you to change the targets, however."
    );
    copy_keeps_division(&PYROTECHNICS);
}
