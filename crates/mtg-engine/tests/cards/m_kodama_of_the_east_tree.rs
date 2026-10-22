//! Kodama of the East Tree (hand-written, `src/cards/kodama_of_the_east_tree.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_permanent_it_put_onto_the_battlefield_doesnt_trigger_it_again() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kodama of the East Tree");
    let ogre = t.hand(P0, "Gray Ogre");
    let bear = t.hand(P0, "Grizzly Bears");
    // A 3-drop enters: put the Ogre (3) in; it doesn't trigger the ability to put the
    // Bears in.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ogre)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
