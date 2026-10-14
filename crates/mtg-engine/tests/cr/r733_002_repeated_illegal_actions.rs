//! CR 733.2, 117.3d: a player whose action was illegal retains priority, but an agent that
//! keeps proposing illegal actions is treated as passing, so the game can't stall on it.

use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_player_who_keeps_attempting_illegal_actions_passes_priority() {
    cr!("733.2", "117.3d");
    let mut t = TestGame::new(2);
    let giant = t.hand(P0, "Hill Giant");
    t.g.turn.passes = 0;
    // Hill Giant without any mana: illegal, over and over.
    for _ in 0..3 {
        t.answer(
            P0,
            DecisionKind::Priority,
            Answer::Action(Action::Cast {
                card: giant,
                method: CastMethod::Normal,
            }),
        );
    }
    for _ in 0..2 {
        t.g.advance();
        assert_eq!(t.g.turn.priority, Some(P0), "the player retains priority");
        assert_eq!(t.g.turn.passes, 0);
    }
    // The third illegal attempt in a row counts as passing.
    t.g.advance();
    assert_eq!(t.g.turn.priority, Some(P1));
    assert_eq!(t.g.turn.passes, 1);
    assert!(t.in_hand(P0, "Hill Giant"));
}
