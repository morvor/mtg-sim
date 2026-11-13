//! Gale, Waterdeep Prodigy (hand-written, `src/cards/gale_waterdeep_prodigy.rs`).

use mtg_engine::testing::*;
use mtg_engine::decision::Decision;
use mtg_engine::*;

#[test]
fn casting_an_instant_lets_you_cast_a_sorcery_from_your_graveyard() {
    cr!("601.2", "603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gale, Waterdeep Prodigy");
    let rot = t.graveyard(P0, "Mind Rot");
    let shock = t.graveyard(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Swamp", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    // Shock (an instant) isn't of the other type: only Mind Rot can be targeted.
    t.answer_targets(P0, &[Entity::Object(shock)]);
    t.settle();
    let mut asked = t.asked();
    asked.retain(|(_, d)| matches!(d, Decision::ChooseTargets { text, .. } if text.contains("other type")));
    let cands = match &asked.last().unwrap().1 {
        Decision::ChooseTargets { candidates, .. } => candidates.clone(),
        _ => vec![],
    };
    assert_eq!(cands, vec![Entity::Object(rot)]);
}

#[test]
fn the_spell_cast_this_way_is_exiled() {
    cr!("601.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gale, Waterdeep Prodigy");
    let rot = t.graveyard(P0, "Mind Rot");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Island");
    t.hand(P1, "Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P0, &[Entity::Object(rot)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_exile("Mind Rot"));
}
