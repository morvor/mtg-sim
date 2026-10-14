//! roundtrip-gaps-3 (docs/agents/items/roundtrip-gaps-3.md): the parser bugs behind the
//! new renderer gap clusters (history conditions, cast history, opt-in choices, one-use
//! replacement effects).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn shape_of_the_wiitigo_looks_at_whether_the_enchanted_creature_attacked() {
    cr!("608.2c", "303.4");
    // "At the beginning of your upkeep, put a +1/+1 counter on enchanted creature if it
    // attacked or blocked since your last upkeep. Otherwise, remove a +1/+1 counter from
    // it." "It" is the enchanted creature, not the Aura (which never attacks).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Shape of the Wiitigo");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3, "{}", t.dump_log());
}

#[test]
fn regna_counts_life_a_teammate_gained() {
    cr!("102.4", "603.4");
    // "At the beginning of each end step, if your team gained life this turn, create two
    // 1/1 white Warrior creature tokens." In Two-Headed Giant, your teammate gaining life
    // is your team gaining life.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Regna, the Redeemer");
    t.set_step(P0, Step::PrecombatMain);
    t.g.gain_life(P1, 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(warriors(&t), 2, "{}", t.dump_log());
}

fn warriors(t: &TestGame) -> usize {
    t.named_on_battlefield("Warrior Token").len()
}

#[test]
fn regna_ignores_an_opponents_life_gain() {
    cr!("102.4");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Regna, the Redeemer");
    t.set_step(P0, Step::PrecombatMain);
    t.g.gain_life(P2, 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(warriors(&t), 0, "{}", t.dump_log());
}
