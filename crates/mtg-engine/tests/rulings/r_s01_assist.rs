//! Rulings batch S01 — assist (CR 702.132): another player may pay for any amount of the
//! generic mana in the spell's total cost.

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

fn untapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| !t.obj_now(**id).tapped).count()
}

#[test]
fn spending_mana_as_though_any_color_doesnt_make_colored_costs_generic() {
    cr!("702.132a", "609.4b");
    ruling!(
        "Skystreamer",
        "If an effect allows a player to cast a spell spending mana “as though it were mana of any color” or “of any type,” that player must still pay for the colored mana in that spell’s total cost. That cost doesn’t become generic."
    );
    supported("Skystreamer");
    // P0 may spend mana as though it were mana of any color (Chromatic Orrery, tapped).
    let mut t = TestGame::new(2);
    let orrery = t.battlefield(P0, "Chromatic Orrery");
    t.g.objects[orrery.0 as usize].tapped = true;
    let mine = t.lands(P0, "Mountain", 1);
    let theirs = t.lands(P1, "Plains", 5);
    // Skystreamer: {4}{W}. P1 assists.
    let s = t.hand(P0, "Skystreamer");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(4));
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.cast(P0, s).go();
    // P1 could pay only the four generic mana; P0 paid {W} with red mana.
    let max: Vec<i64> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseNumber { max, .. } if *p == P1 => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(max, vec![4]);
    assert_eq!(untapped(&t, &mine), 0);
    assert_eq!(untapped(&t, &theirs), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Skystreamer").len(), 1);
    // Without mana of their own, P0 can't cast it, however much P1 could pay.
    let mut t = TestGame::new(2);
    let orrery = t.battlefield(P0, "Chromatic Orrery");
    t.g.objects[orrery.0 as usize].tapped = true;
    t.lands(P1, "Plains", 6);
    let s = t.hand(P0, "Skystreamer");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(4));
    assert!(t.cast(P0, s).try_go().is_err());
    assert!(t.in_hand(P0, "Skystreamer"));
}
