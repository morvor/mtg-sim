//! Swirl the Mists (hand-written, `src/cards/swirl_the_mists.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn color_words_become_the_chosen_word() {
    cr!("612.2");
    ruling!("Swirl the Mists", "also affects spells that are cast and permanents that enter after");
    ruling!("Swirl the Mists", "It ends as soon as Swirl the Mists leaves the battlefield");
    let mut t = TestGame::new(2);
    // "White creatures get +1/+1."
    t.battlefield(P0, "Crusade");
    let lions = t.battlefield(P0, "Savannah Lions");
    // Red (white, blue, black, red, green).
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let swirl = t.enter(P0, "Swirl the Mists");
    t.resolve_all();
    // Entering afterwards.
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(goblin), (2, 2));
    assert_eq!(t.pt(lions), (2, 1));
    t.g.destroy(swirl, None);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(goblin), (1, 1));
    assert_eq!(t.pt(lions), (3, 2));
}
