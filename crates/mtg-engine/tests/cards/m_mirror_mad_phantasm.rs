//! Mirror-Mad Phantasm (hand-written, `src/cards/mirror_mad_phantasm.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn it_returns_and_mills_the_cards_revealed_before_it() {
    cr!("701.20a");
    ruling!("Mirror-Mad Phantasm", "its owner reveals cards until they reveal a card named Mirror-Mad Phantasm");
    let mut t = TestGame::new(2);
    let phantasm = t.battlefield(P0, "Mirror-Mad Phantasm");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Island", 2);
    let lib = t.g.players[0].library.len();
    t.activate(P0, phantasm, 0, &[]).unwrap();
    t.resolve_all();
    let back = t.named_on_battlefield("Mirror-Mad Phantasm");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], phantasm);
    let milled = t.g.players[0].graveyard.len();
    assert_eq!(t.g.players[0].library.len() + milled, lib);
}
