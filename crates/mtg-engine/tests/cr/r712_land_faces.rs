//! CR 712.12: a player playing a modal double-faced card as a land chooses one of its
//! faces that's a land; it enters with that face up.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

const PATHWAY: &str = "Clearwater Pathway // Murkwater Pathway";

#[test]
fn a_modal_double_faced_card_with_two_land_faces_is_played_with_the_chosen_face() {
    cr!("712.12");
    for (choice, name) in [(0, "Clearwater Pathway"), (1, "Murkwater Pathway")] {
        let mut t = TestGame::new(2);
        let card = t.hand(P0, PATHWAY);
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.play_land(P0, card).unwrap();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
    }
}
