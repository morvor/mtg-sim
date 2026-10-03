//! CR 107.3d: X in the cost of a special action.

use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn x_in_a_special_action_cost_is_chosen_immediately_before_paying_it() {
    cr!("107.3d");
    let mut t = TestGame::new(2);
    // Warbreak Trumpeter: "Morph {X}{X}{R}". Turning it face up is a special action.
    let trumpeter = t.battlefield(P0, "Warbreak Trumpeter");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, trumpeter));
    t.g.recompute();
    let mountains = t.lands(P0, "Mountain", 5);
    let turn_up = Action::Special(SpecialAction::TurnFaceUp { obj: trumpeter });
    assert!(mtg_engine::keyword_impls::special_actions(&t.g, P0).contains(&turn_up));
    let asked_before = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.g.perform_action(P0, turn_up.clone()).unwrap();
    // P0 chose X as they took the action, then paid {2}{2}{R}.
    assert!(t.asked()[asked_before..].iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseX { source, .. } if *source == trumpeter)));
    assert!(mountains.iter().all(|m| t.obj(*m).tapped));
    assert!(!t.obj(trumpeter).face_down);
    assert_eq!(t.obj(trumpeter).chars.name.as_str(), "Warbreak Trumpeter");
}

#[test]
fn a_special_action_x_that_cant_be_paid_leaves_the_permanent_face_down() {
    cr!("107.3d");
    let mut t = TestGame::new(2);
    let trumpeter = t.battlefield(P0, "Warbreak Trumpeter");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, trumpeter));
    t.g.recompute();
    let mountains = t.lands(P0, "Mountain", 3);
    // X = 2 would cost {2}{2}{R}: five mana, more than P0 has.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let r = t.g.perform_action(
        P0,
        Action::Special(SpecialAction::TurnFaceUp { obj: trumpeter }),
    );
    assert!(r.is_err());
    assert!(t.obj(trumpeter).face_down);
    assert!(mountains.iter().all(|m| !t.obj(*m).tapped));
    // X = 1 costs {1}{1}{R}.
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::TurnFaceUp { obj: trumpeter }),
    )
    .unwrap();
    assert!(!t.obj(trumpeter).face_down);
}
