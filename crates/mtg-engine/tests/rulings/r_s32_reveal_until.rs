//! Rulings batch S32 — Balustrade Spy ("When this creature enters, target player reveals
//! cards from the top of their library until they reveal a land card, then puts those
//! cards into their graveyard."): revealing until a land card (CR 701.20a), which may be
//! the whole library.

use crate::r_s01_common::*;
use crate::r_s04_common::graveyard_names;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn with_no_land_card_the_whole_library_is_revealed_and_put_into_the_graveyard() {
    cr!("701.20a");
    ruling!(
        "Balustrade Spy",
        "If the target player has no land cards in their library, all cards from that library will be revealed and put into their graveyard."
    );
    supported("Balustrade Spy");
    supported("Undercity Informer");
    // P1's library: Grizzly Bears, Shock, Forest on top of thirty cards: the three go.
    let mut t = TestGame::new(2);
    let library = t.library_size(P1);
    stack_library(&mut t, P1, &["Grizzly Bears", "Shock", "Forest"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Balustrade Spy");
    t.resolve_all();
    let mut gy = graveyard_names(&t, P1);
    gy.sort();
    assert_eq!(gy, vec!["Forest", "Grizzly Bears", "Shock"]);
    assert_eq!(t.library_size(P1), library);
    // A library without land cards: all of it.
    let mut t = TestGame::new(2);
    let library = t.library_size(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Balustrade Spy");
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), library);
    // Undercity Informer: "{1}, Sacrifice a creature: Target player reveals cards ..." —
    // P0 targets themself.
    let mut t = TestGame::new(2);
    let informer = t.battlefield(P0, "Undercity Informer");
    t.lands(P0, "Swamp", 1);
    stack_library(&mut t, P0, &["Shock", "Swamp"]);
    t.activate(P0, informer, 0, &[Entity::Player(P0)])
        .expect("activate Undercity Informer");
    t.resolve_all();
    let mut gy = graveyard_names(&t, P0);
    gy.sort();
    assert_eq!(gy, vec!["Shock", "Swamp", "Undercity Informer"]);
}
