//! Camouflage (hand-written, `src/cards/camouflage.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn blockers_go_into_piles_assigned_at_random() {
    cr!("509.1", "700.3");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Gray Ogre");
    t.battlefield(P1, "Wall of Stone");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1)), (b, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.lands(P0, "Forest", 1);
    let c = t.hand(P0, "Camouflage");
    t.cast(P0, c).go();
    t.resolve_all();
    // The Wall goes into pile 1; the blockers P1 tries to declare are ignored.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.advance_to(P0, Step::EndOfCombat);
    // Exactly one attacker was blocked by the Wall.
    assert_eq!(t.life(P1), 20 - 2);
}
