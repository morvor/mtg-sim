//! General Jarkeld (hand-written, `src/cards/general_jarkeld.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn switches_the_blockers_of_two_attackers() {
    cr!("509.1", "506.4");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jarkeld = t.battlefield(P1, "General Jarkeld");
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(wurm, Entity::Player(P1)), (bears, Entity::Player(P1))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, wurm), (wall, bears)]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P1, jarkeld, 0, &[Entity::Object(wurm), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // The Giant now blocked the Bears (and killed them); the Wall blocked the Wurm.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wall).damage, 6);
}

#[test]
fn nothing_changes_if_a_blocker_couldnt_block_the_other() {
    cr!("509.1b");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jarkeld = t.battlefield(P1, "General Jarkeld");
    let hawk = t.battlefield(P1, "Wind Drake");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(angel, Entity::Player(P1)), (bears, Entity::Player(P1))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(hawk, angel), (giant, bears)]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P1, jarkeld, 0, &[Entity::Object(angel), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // The Giant can't block the flying Angel: no switch.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Wind Drake"));
}
