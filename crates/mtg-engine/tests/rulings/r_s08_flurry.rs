//! Rulings batch S08 — flurry (an ability word, CR 207.2c): "Whenever you cast your second
//! spell each turn, ...". Spells cast before the flurry permanent was on the battlefield
//! count (CR 603.2).

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_flurry_permanent_cast_first_makes_the_next_spell_the_second() {
    cr!("207.2c", "603.2");
    ruling!(
        "Devoted Duelist",
        "Spells that were cast before a permanent with flurry count. If that permanent was the first spell you cast that turn, the next spell you cast that turn is your second spell."
    );
    supported("Devoted Duelist");
    // Devoted Duelist: "Flurry — Whenever you cast your second spell each turn, this
    // creature deals 1 damage to each opponent."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let duelist = t.hand(P0, "Devoted Duelist");
    t.cast(P0, duelist).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P0).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // The third spell doesn't trigger it.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P0).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_spell_cast_before_the_flurry_permanent_counts_as_the_first() {
    cr!("207.2c", "603.2");
    ruling!(
        "Wingblade Disciple",
        "Spells that were cast before a permanent with flurry count."
    );
    supported("Wingblade Disciple");
    // Wingblade Disciple: "Flurry — Whenever you cast your second spell each turn, create
    // a 1/1 white Bird creature token with flying."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    // The Disciple is put onto the battlefield after P0's first spell (without being
    // cast); P0's next spell is its second.
    let disciple = t.battlefield(P0, "Wingblade Disciple");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(disciple).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 1);
}
