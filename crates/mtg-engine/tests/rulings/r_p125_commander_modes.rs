//! Rulings batch P125 — "Choose one. If you control a commander [as you cast this spell],
//! you may choose both instead." (CR 700.2, 903.3): any commander you control counts, and
//! more commanders give nothing more.

use crate::r_p125_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield as `owner`'s commander, under P0's
/// control.
fn commander_for_p0(t: &mut TestGame, owner: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(owner, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.dirty = true;
    if owner != P0 {
        crate::r_s06_common::give_control(t, id, P0);
    }
    t.settle();
    t.g.current(id)
}

/// The most modes P0 was offered to choose since decision `from`.
fn max_modes(t: &TestGame, from: usize) -> Vec<u32> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseModes { max, .. } if *p == P0 => Some(*max),
            _ => None,
        })
        .collect()
}

/// P0 casts Akroma's Will choosing both modes if offered; returns the most modes offered.
fn akromas_will(t: &mut TestGame) -> Vec<u32> {
    supported("Akroma's Will");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    cast_new(t, P0, "Akroma's Will", &[]);
    t.resolve_all();
    max_modes(t, from)
}

#[test]
fn the_commander_you_control_doesnt_have_to_be_yours() {
    cr!("700.2", "903.3");
    ruling!("Akroma's Will", "The commander you control doesn't have to be your commander.");
    ruling!(
        "SOLDIER Military Program",
        "The commander you control doesn't have to be your commander."
    );
    // Akroma's Will ("If you control a commander as you cast this spell"), with P1's
    // commander under P0's control: both modes.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    commander_for_p0(&mut t, P1, "Hill Giant");
    assert_eq!(akromas_will(&mut t), vec![2]);
    assert!(has(&t, bears, KeywordKind::Flying));
    assert!(indestructible(&t, bears));
    // Without a commander: one mode.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(akromas_will(&mut t), vec![1]);

    // SOLDIER Military Program ("If you control a commander"): a trigger at the beginning
    // of combat.
    supported("SOLDIER Military Program");
    for with_commander in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "SOLDIER Military Program");
        if with_commander {
            commander_for_p0(&mut t, P1, "Hill Giant");
        }
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Modes, Answer::Indices(if with_commander { vec![0, 1] } else { vec![0] }));
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert_eq!(max_modes(&t, from), vec![if with_commander { 2 } else { 1 }]);
        let soldiers = crate::r_p108_common::tokens_with(&t, P0, "Soldier");
        assert_eq!(soldiers, 1);
    }
}

#[test]
fn more_commanders_give_no_extra_bonus() {
    cr!("700.2", "903.3");
    ruling!("Akroma's Will", "There's no extra bonus if you control more than one commander.");
    let mut t = TestGame::new(2);
    commander_for_p0(&mut t, P0, "Grizzly Bears");
    commander_for_p0(&mut t, P1, "Hill Giant");
    assert_eq!(akromas_will(&mut t), vec![2]);
}
