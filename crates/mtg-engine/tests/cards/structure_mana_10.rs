//! Structure-coverage tests (round 10): mana-ability structures shared by several cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{act, untapped};
use super::structure_seed_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId, m: ManaType) -> u32 {
    t.g.player(p).mana_pool.count(m) as u32
}

fn pool_size(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.mana.len()
}

#[test]
fn bog_initiate_filters_generic_mana_into_black() {
    cr!("605.1a", "605.3b", "118.3");
    // "{1}: Add {B}."
    supported("Bog Initiate");
    let mut t = TestGame::new(2);
    let b = t.battlefield_sick(P0, "Bog Initiate");
    assert!(act(&mut t, P0, b, "add {b}", &[]).is_err(), "nothing to pay {{1}} with");
    t.clear_answers();
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, b, "add {b}", &[]).unwrap();
    assert_eq!(t.stack_len(), 0, "a mana ability doesn't use the stack");
    assert_eq!(untapped(&t, P0, "Wastes"), 1, "{{1}} paid");
    assert!(!t.obj_now(b).tapped, "no {{T}}: summoning sickness doesn't matter");
    assert_eq!(pool(&t, P0, ManaType::B), 1);
    assert_eq!(pool_size(&t, P0), 1, "the {{C}} was spent");
    t.g.player_mut(P0).mana_pool.mana.clear();
    act(&mut t, P0, b, "add {b}", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::B), 1, "it can be activated again");
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
}

#[test]
fn cascading_cataracts_adds_five_mana_in_any_combination_of_colors() {
    cr!("605.1a", "106.1b", "118.3");
    // "{5}, {T}: Add five mana in any combination of colors."
    supported("Cascading Cataracts");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cascading Cataracts");
    t.lands(P0, "Wastes", 4);
    assert!(act(&mut t, P0, c, "five mana", &[]).is_err(), "{{5}} needs five mana");
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    // W, U, B, R, G: two white, one blue, two green.
    for i in [0, 0, 1, 4, 4] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    act(&mut t, P0, c, "five mana", &[]).unwrap();
    assert!(t.obj_now(c).tapped);
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_size(&t, P0), 5);
    assert_eq!(pool(&t, P0, ManaType::W), 2);
    assert_eq!(pool(&t, P0, ManaType::U), 1);
    assert_eq!(pool(&t, P0, ManaType::G), 2);
    assert_eq!(pool(&t, P0, ManaType::C), 0, "colors only");
}
