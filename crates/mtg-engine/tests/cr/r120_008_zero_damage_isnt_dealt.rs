//! CR 120.8, 616.1: once a replacement effect reduces damage to 0, no damage is dealt, so
//! an effect that modifies how much damage a source deals doesn't apply afterward. The
//! affected player chooses the order of the replacement effects.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 (with Torbran, "a red source you control ... deals that much damage plus 2") casts
/// Spark Jolt (1 damage) at P1, who has Benevolent Unicorn ("a spell ... deals that much
/// damage minus 1") and applies the effect at `first` first. Returns P1's life and the
/// effect P1 chose first.
fn jolt(first: usize) -> (i32, String) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Torbran, Thane of Red Fell");
    t.battlefield(P1, "Benevolent Unicorn");
    t.lands(P0, "Mountain", 1);
    let jolt = t.hand(P0, "Spark Jolt");
    t.answer(P1, DecisionKind::Replacement, Answer::Index(first));
    let from = t.asked().len();
    t.cast(P0, jolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    let chosen = t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } if *p == P1 => {
                Some(options[first.min(options.len() - 1)].clone())
            }
            _ => None,
        })
        .expect("P1 chose the order");
    (t.life(P1), chosen)
}

#[test]
fn damage_reduced_to_zero_isnt_increased_afterward() {
    cr!("120.8", "616.1");
    for first in 0..2 {
        let (life, chosen) = jolt(first);
        if chosen.contains("Unicorn") {
            // 1 - 1 = 0: no damage is dealt, Torbran has nothing to modify.
            assert_eq!(life, 20, "{chosen}");
        } else {
            // 1 + 2 = 3, then 3 - 1 = 2.
            assert_eq!(life, 18, "{chosen}");
        }
    }
}
