//! Mathemagics (hand-written, `src/cards/mathemagics.rs`): draws 2^X cards.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn draws_two_to_the_x() {
    cr!("121.2");
    for (x, n) in [(0, 1), (1, 2), (3, 8)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 2 + 2 * x as usize);
        let m = t.hand(P0, "Mathemagics");
        t.cast(P0, m).x(x).target(P1).go();
        t.resolve();
        assert_eq!(t.hand_size(P1), n, "X = {x}");
    }
}
