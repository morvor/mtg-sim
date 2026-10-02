//! In-game tests for parser bugs found while teaching the Oracle round-trip renderer
//! (`oracle/render/`) the AST nodes it couldn't put into words.

use mtg_engine::card::card;
use mtg_engine::oracle::render::compare::check_card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_round_trips(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
    let r = check_card(&c);
    assert!(
        r.pass,
        "{name} doesn't round-trip:\n  oracle: {:?}\n  rendered: {:?}\n  gaps: {:?}",
        r.unmatched_oracle, r.unmatched_rendered, r.gaps
    );
}

/// "When this creature enters, each player discards a card. If you discarded a card this
/// way, draw a card." Only a card you discarded counts: when only an opponent discards
/// (your hand is empty), you don't draw. It used to count any player's discarded card.
#[test]
fn fanatic_of_the_harrowing_counts_only_your_discard() {
    cr!("608.2c", "701.9a");
    assert_round_trips("Fanatic of the Harrowing");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.enter(P0, "Fanatic of the Harrowing");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the opponent discarded");
    assert_eq!(t.library_size(P0), lib, "you discarded nothing, so you don't draw");

    // With a card in your hand, you discard it and draw.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.enter(P0, "Fanatic of the Harrowing");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), lib - 1, "you discarded a card, so you draw");
}
