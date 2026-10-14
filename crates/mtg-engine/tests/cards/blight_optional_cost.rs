//! Spells with "As an additional cost to cast this spell, you may blight N." whose text
//! refers back to it, "if this spell's additional cost was paid" (patterns in
//! `src/oracle/patterns/a701_blight_optional.rs`).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The (min, max) number of modes P0 was offered to choose.
fn mode_counts(t: &TestGame) -> Vec<(u32, u32)> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseModes { min, max, .. } if *p == P0 => Some((*min, *max)),
            _ => None,
        })
        .collect()
}

#[test]
fn blight_optional_cost_cards_compile() {
    assert_compiles(&["Pyrrhic Strike", "Requiting Hex", "Burning Curiosity"]);
}

#[test]
fn pyrrhic_strike_chooses_both_modes_if_its_blight_cost_was_paid() {
    cr!("700.2", "601.2b", "701.68a", "118.8a");
    // "As an additional cost to cast this spell, you may blight 2. Choose one. If this
    // spell's additional cost was paid, choose both instead. • Destroy target artifact or
    // enchantment. • Destroy target creature with mana value 3 or greater." ({2}{W})
    for blight in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let stone = t.battlefield(P1, "Mind Stone");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Plains", 3);
        let c = t.hand(P0, "Pyrrhic Strike");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(blight));
        t.answer_choose(P0, &[Entity::Object(bears)]);
        let modes: &[usize] = if blight { &[0, 1] } else { &[1] };
        let mut b = t.cast(P0, c).modes(modes);
        if blight {
            b = b.target(stone);
        }
        b.target(giant).go();
        t.resolve_all();
        let n = if blight { 2 } else { 1 };
        assert_eq!(mode_counts(&t), vec![(n, n)], "blighted: {blight}");
        assert!(t.in_graveyard(P1, "Hill Giant"));
        assert_eq!(t.in_graveyard(P1, "Mind Stone"), blight);
        // Blighting 2 put two -1/-1 counters on the Bears.
        assert_eq!(t.in_graveyard(P0, "Grizzly Bears"), blight);
        if !blight {
            assert_eq!(t.counters(bears, counters::MINUS1), 0);
        }
    }
}

#[test]
fn requiting_hex_gains_life_if_its_blight_cost_was_paid() {
    cr!("701.68a", "608.2c");
    // "As an additional cost to cast this spell, you may blight 1. Destroy target creature
    // with mana value 2 or less. If this spell's additional cost was paid, you gain 2
    // life." ({B})
    for blight in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 1);
        let c = t.hand(P0, "Requiting Hex");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(blight));
        t.answer_choose(P0, &[Entity::Object(giant)]);
        t.cast(P0, c).target(bears).go();
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        assert_eq!(t.counters(giant, counters::MINUS1), u32::from(blight));
        assert_eq!(t.life(P0), if blight { 22 } else { 20 }, "blighted: {blight}");
    }
}
