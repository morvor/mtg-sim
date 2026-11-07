//! "Create a 0/0 ... token. Put X +1/+1 counters on it, where X is [value]": "it" is the
//! token just created, and X is defined by the text (CR 107.3c), with the value's own
//! referents ("that spell's mana value") unchanged.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(name: &str) {
    let u = card(name).unsupported_text().join(" | ");
    assert!(u.is_empty(), "{name} has unsupported text: {u}");
}

#[test]
fn magecraft_fractal_gets_counters_equal_to_that_spells_mana_value() {
    // Deekah, Fractal Theorist: "Magecraft — Whenever you cast or copy an instant or
    // sorcery spell, create a 0/0 green and blue Fractal creature token. Put X +1/+1
    // counters on it, where X is that spell's mana value."
    cr!("107.3c");
    assert_compiles("Deekah, Fractal Theorist");
    let mut t = TestGame::new(2);
    let deekah = t.battlefield(P0, "Deekah, Fractal Theorist");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination"); // {2}{U}
    t.cast(P0, div).go();
    t.settle();
    t.resolve(); // the magecraft trigger
    let fractals = t.named_on_battlefield("Fractal Token");
    assert_eq!(fractals.len(), 1);
    assert_eq!(t.counters(fractals[0], "+1/+1"), 3);
    assert_eq!(t.pt(fractals[0]), (3, 3));
    assert_eq!(t.counters(deekah, "+1/+1"), 0);
}

#[test]
fn fractal_gets_counters_equal_to_cards_in_hand() {
    // Manifestation Sage: "When this creature enters, create a 0/0 green and blue Fractal
    // creature token. Put X +1/+1 counters on it, where X is the number of cards in your
    // hand."
    cr!("107.3c");
    assert_compiles("Manifestation Sage");
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.hand(P0, "Island");
    }
    let hand = t.hand_size(P0) as u32;
    let sage = t.enter(P0, "Manifestation Sage");
    t.resolve_all();
    let fractals = t.named_on_battlefield("Fractal Token");
    assert_eq!(fractals.len(), 1);
    assert_eq!(t.counters(fractals[0], "+1/+1"), hand);
    assert_eq!(t.counters(sage, "+1/+1"), 0);
}
