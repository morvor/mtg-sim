//! Minamo's Meddling (hand-written, `src/cards/minamos_meddling.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn discards_the_cards_with_the_names_of_the_spliced_cards() {
    cr!("702.47a", "701.6a");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let spike = t.hand(P1, "Lava Spike");
    t.hand(P1, "Glacial Ray");
    t.hand(P1, "Glacial Ray");
    t.hand(P1, "Grizzly Bears");
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    let spell = t.cast(P1, spike).target(P0).target(P0).go();
    t.lands(P0, "Island", 4);
    let mm = t.hand(P0, "Minamo's Meddling");
    t.cast(P0, mm).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Lava Spike"));
    // Both Glacial Rays were discarded (the spliced one and the other); not the Bears.
    assert!(!t.in_hand(P1, "Glacial Ray"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn nothing_is_discarded_without_splices() {
    cr!("701.6a");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let spike = t.hand(P1, "Lava Spike");
    t.hand(P1, "Glacial Ray");
    let spell = t.cast(P1, spike).target(P0).go();
    t.lands(P0, "Island", 4);
    let mm = t.hand(P0, "Minamo's Meddling");
    t.cast(P0, mm).target(spell).go();
    t.resolve();
    assert!(t.in_hand(P1, "Glacial Ray"));
}
