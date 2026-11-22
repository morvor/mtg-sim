//! Rulings batch P003 — damage divided as you choose among one, two, or three targets:
//! the number of targets and the division are chosen as the spell is cast or the
//! triggered ability is put on the stack, each target getting at least 1 (CR 601.2d,
//! 603.3d); the division doesn't change when some targets become illegal, and nothing
//! resolves when all are (CR 608.2b).

use crate::r_p003_common::*;
use crate::r_p109_common::division_of;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The divided-damage cards: name, total damage, and how it gets on the stack.
#[derive(Clone, Copy, PartialEq)]
enum How {
    Cast(&'static str),
    Enters,
    Dies,
}

const CARDS: [(&str, u32, How); 5] = [
    ("Arc Lightning", 3, How::Cast("2R")),
    ("Flames of the Firebrand", 3, How::Cast("2R")),
    ("Forked Lightning", 4, How::Cast("3R")),
    ("Gandalf, Spark Starter", 3, How::Enters),
    ("Gang of Devils", 3, How::Dies),
];

/// P0 casts the spell or makes the ability trigger with the targets and division queued;
/// returns the object on the stack (the spell or the triggered ability).
fn on_stack(t: &mut TestGame, name: &str, how: How, targets: &[Entity], div: &[i64]) -> ObjectId {
    supported(name);
    t.answer_targets(P0, targets);
    if !div.is_empty() {
        divide(t, P0, div);
    }
    match how {
        How::Cast(cost) => {
            let c = t.hand(P0, name);
            mana(t, P0, cost);
            t.cast(P0, c).go();
        }
        How::Enters => {
            enter(t, P0, name);
        }
        How::Dies => {
            let d = t.battlefield(P0, name);
            kill(t, d);
        }
    }
    t.settle();
    t.clear_answers();
    *t.g.stack.last().expect("on the stack")
}

/// Three Colossal Dreadmaws (6/6) for P1.
fn wurms(t: &mut TestGame) -> Vec<ObjectId> {
    (0..3)
        .map(|_| t.battlefield(P1, "Colossal Dreadmaw"))
        .collect()
}

fn dmgs(t: &TestGame, ws: &[ObjectId]) -> Vec<u32> {
    ws.iter().map(|w| dmg(t, *w)).collect()
}

#[test]
fn the_targets_and_division_are_chosen_as_it_goes_on_the_stack() {
    cr!("601.2c", "601.2d", "603.3d");
    ruling!(
        "Arc Lightning",
        "You choose how many targets Arc Lightning has and how the damage is divided as you cast the spell. Each target must receive at least 1 damage."
    );
    ruling!(
        "Gandalf, Spark Starter",
        "You choose how many targets Gandalf's triggered ability has and how the damage is divided as you put the ability on the stack. Each target must receive at least 1 damage."
    );
    ruling!(
        "Flames of the Firebrand",
        "You divide the damage as you cast Flames of the Firebrand, not as it resolves. Each target must be assigned at least 1 damage. (In other words, as you put the spell on the stack, you choose whether to have it deal 3 damage to a single target, 2 damage to one target and 1 damage to another target, or 1 damage to each of three targets.)"
    );
    ruling!(
        "Forked Lightning",
        "You divide the damage as you cast Forked Lightning, not as it resolves. Each target you choose must be assigned at least 1 damage."
    );
    ruling!(
        "Gang of Devils",
        "You divide the damage as you put Gang of Devils’s triggered ability on the stack, not as it resolves. Each target must be assigned at least 1 damage. (In other words, as you put the ability on the stack, you choose whether to have it deal 3 damage to a single target, 2 damage to one target and 1 damage to another target, or 1 damage to each of three targets.)"
    );
    for (name, total, how) in CARDS {
        // One, two and three targets.
        for n in 1..=3usize {
            let mut t = TestGame::new(2);
            let ws = wurms(&mut t);
            let es: Vec<Entity> = ws[..n].iter().map(|w| obj(*w)).collect();
            let mut div = vec![1i64; n];
            div[0] = total as i64 - (n as i64 - 1);
            let s = on_stack(&mut t, name, how, &es, &div);
            let want: Vec<u32> = div.iter().map(|d| *d as u32).collect();
            if n > 1 {
                assert_eq!(division_of(&t, s), want, "{name}: {n} targets");
            }
            t.resolve_all();
            let mut got = dmgs(&t, &ws);
            got.truncate(n);
            assert_eq!(got, want, "{name}: {n} targets");
        }
        // A division that gives a target 0 isn't accepted.
        let mut t = TestGame::new(2);
        let ws = wurms(&mut t);
        let es = [obj(ws[0]), obj(ws[1])];
        let s = on_stack(&mut t, name, how, &es, &[0, total as i64]);
        let div = division_of(&t, s);
        assert_eq!(div.iter().sum::<u32>(), total, "{name}: {div:?}");
        assert!(div.iter().all(|d| *d >= 1), "{name}: {div:?}");
    }
}

#[test]
fn illegal_targets_keep_the_original_division_and_all_illegal_doesnt_resolve() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Arc Lightning",
        "If some of the creatures are illegal targets as Arc Lightning tries to resolve, the original division of damage still applies but no damage is dealt to the illegal targets. If all targets are illegal, Arc Lightning doesn’t resolve."
    );
    ruling!(
        "Gandalf, Spark Starter",
        "If some of the targets are illegal targets as Gandalf's triggered ability tries to resolve, the original division of damage still applies but no damage is dealt to the illegal targets. If all targets are illegal, Gandalf's triggered ability doesn't resolve."
    );
    for name in ["Arc Lightning", "Gandalf, Spark Starter"] {
        let how = CARDS.iter().find(|c| c.0 == name).unwrap().2;
        // 2 to the first Dreadmaw, 1 to the second, which leaves.
        let mut t = TestGame::new(2);
        let ws = wurms(&mut t);
        on_stack(&mut t, name, how, &[obj(ws[0]), obj(ws[1])], &[2, 1]);
        kill(&mut t, ws[1]);
        t.resolve_all();
        assert_eq!(dmg(&t, ws[0]), 2, "{name}");
        // Both leave: nothing is dealt damage.
        let mut t = TestGame::new(2);
        let ws = wurms(&mut t);
        let s = on_stack(&mut t, name, how, &[obj(ws[0]), pl(P1)], &[2, 1]);
        kill(&mut t, ws[0]);
        crate::r_p007_common::player_hexproof(&mut t, P1);
        t.resolve_all();
        assert!(!t.g.stack.contains(&s));
        assert_eq!(t.life(P1), 20, "{name}");
        assert!(
            crate::r_p034_common::damage_events(&t).is_empty(),
            "{name}: no damage"
        );
    }
}
