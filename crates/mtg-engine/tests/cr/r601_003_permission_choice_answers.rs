//! CR 601.2, 601.3: the player announces which permission they're using to cast a card
//! from outside their hand. An answer that names no permission among those offered gets
//! the default (the first one whose costs could be paid), never another permission.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_out_of_range_permission_answer_uses_the_default_permission() {
    cr!("601.2", "601.3", "110.4");
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let thopter = t.graveyard(P0, "Ornithopter");
    // Muldrotha offers its artifact and its creature permission for the artifact creature
    // card (asked by name: both come from Muldrotha); P0 answers neither.
    t.answer(P0, DecisionKind::Option, Answer::Index(7));
    t.cast(P0, thopter).go();
    assert_eq!(
        t.g.history.once_permissions_used,
        vec![(muldrotha, "artifact spell".into())]
    );
}
