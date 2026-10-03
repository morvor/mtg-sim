//! Rulings batch S17 — triple (City on Fire): dividing damage before it's tripled.

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn divided_damage_is_divided_before_its_tripled() {
    cr!("601.2d", "614.1a", "120.4");
    ruling!(
        "City on Fire",
        "If an effect such as that of Chandra's Pyrohelix asks you to divide damage among targets, you must divide the unmodified damage before tripling it."
    );
    supported("City on Fire");
    supported("Chandra's Pyrohelix");
    // City on Fire: "If a source you control would deal damage to a permanent or player,
    // it deals triple that damage instead." Chandra's Pyrohelix: "deals 2 damage divided
    // as you choose among one or two targets."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "City on Fire");
    let giants = [t.battlefield(P1, "Hill Giant"), t.battlefield(P1, "Hill Giant")];
    t.lands(P0, "Mountain", 2);
    let helix = t.hand(P0, "Chandra's Pyrohelix");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast(P0, helix)
        .targets(&[Entity::Object(giants[0]), Entity::Object(giants[1])])
        .go();
    // The 2 damage was divided, not 6.
    let totals: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { total, .. } => Some(*total),
            _ => None,
        })
        .collect();
    assert_eq!(totals, vec![2]);
    t.resolve_all();
    // 1 each, tripled: 3 damage to each 3/3 Hill Giant.
    assert!(!t.on_battlefield(giants[0]));
    assert!(!t.on_battlefield(giants[1]));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Divided 2 to one Giant and 0 is impossible with two targets; with one target it gets
    // all 2, tripled to 6 (a player: 20 - 6).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "City on Fire");
    t.lands(P0, "Mountain", 2);
    let helix = t.hand(P0, "Chandra's Pyrohelix");
    t.cast(P0, helix).targets(&[Entity::Player(P1)]).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}
