//! CR 708.3 / 400.6 audit (zones.rs, `own_move_replacements`): an object put onto the
//! battlefield face down is turned face down before it enters, so its own "if this would
//! enter" replacement ability has no effect; put onto the battlefield face up, the same
//! ability applies to the move (400.6: the object's own abilities that would affect it).

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

#[test]
fn a_manifested_card_enters_without_its_own_enters_replacement() {
    cr!("708.3", "701.40a", "400.6");
    supported("Soul Summons");
    supported("Mox Diamond");
    let mut t = TestGame::new(2);
    // Mox Diamond: "If this artifact would enter, you may discard a land card instead. If
    // you do, put this artifact onto the battlefield. If you don't, put it into its owner's
    // graveyard." Manifested (Soul Summons: "Manifest the top card of your library."), it's
    // a face-down 2/2 as it enters: P0 isn't asked to discard the Forest, and it doesn't go
    // to the graveyard.
    let mox = t.library_top(P0, "Mox Diamond");
    t.hand(P0, "Forest");
    t.lands(P0, "Plains", 2);
    t.answer_yes(P0, true);
    let summons = t.hand(P0, "Soul Summons");
    t.cast(P0, summons).go();
    t.resolve_all();
    let m = t.g.current(mox);
    assert!(t.on_battlefield(m), "manifested");
    assert!(t.obj_now(m).face_down);
    assert_eq!(t.pt(m), (2, 2));
    assert!(t.in_hand(P0, "Forest"));
    assert!(!t.in_graveyard(P0, "Mox Diamond"));
    // Put onto the battlefield face up, its replacement applies: with no land card in hand
    // to discard, it goes to the graveyard instead.
    let mut t = TestGame::new(2);
    let mox = t.hand(P0, "Mox Diamond");
    t.cast(P0, mox).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mox Diamond"));
    assert!(t.named_on_battlefield("Mox Diamond").is_empty());
}
