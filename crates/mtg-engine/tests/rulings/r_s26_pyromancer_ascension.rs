//! Rulings batch S26 — Pyromancer Ascension: quest counters for spells with the same name
//! as a card in your graveyard, and copies of instants and sorceries cast while it has
//! two or more (a condition of the trigger event, CR 603.2), which keep the original's
//! mode (CR 707.10, 700.2g).

use crate::r_s01_common::supported;
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn pyromancer_ascension_counts_spells_named_like_a_graveyard_card() {
    cr!("603.2", "201.2");
    supported("Pyromancer Ascension");
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Pyromancer Ascension");
    t.lands(P0, "Mountain", 2);
    // No Lightning Bolt in the graveyard: no counter.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(asc, "quest"), 0);
    // Now the first Bolt is there: the second one gets a counter.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(asc, "quest"), 1);
}

#[test]
fn pyromancer_ascension_copy_keeps_the_mode() {
    cr!("707.10", "700.2g", "603.2");
    ruling!(
        "Pyromancer Ascension",
        "If the spell that’s copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. A different mode cannot be chosen."
    );
    supported("Pyromancer Ascension");
    supported("Boros Charm");
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Pyromancer Ascension");
    t.lands(P0, "Plateau", 4);
    // With one quest counter, nothing is copied.
    t.g.add_counters(Entity::Object(asc), "quest", 1, None);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // With two, Boros Charm is copied with the same mode (and the copy's controller isn't
    // asked for modes).
    t.g.add_counters(Entity::Object(asc), "quest", 1, None);
    let charm = t.hand(P0, "Boros Charm");
    let charm = t.cast(P0, charm).modes(&[0]).target(P1).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![2]),
    );
    t.settle();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(modes_on_stack(&t, copies[0]), modes_on_stack(&t, charm));
    t.resolve_all();
    assert_eq!(t.life(P1), 9);
}
