//! Structure-coverage tests (round 6): mana-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::act;
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
fn tainted_isle_adds_two_colors_only_with_a_swamp() {
    cr!("605.1a", "605.3b", "602.1b");
    // "{T}: Add {U} or {B}. Activate only if you control a Swamp."
    supported("Tainted Isle");
    let mut t = TestGame::new(2);
    let isle = t.battlefield(P0, "Tainted Isle");
    t.battlefield(P1, "Swamp");
    t.battlefield(P0, "Island");
    assert!(
        act(&mut t, P0, isle, "add {u} or {b}", &[]).is_err(),
        "you control no Swamp"
    );
    t.clear_answers();
    // The colorless ability has no condition.
    act(&mut t, P0, isle, "add {c}", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::C), 1);
    t.g.player_mut(P0).mana_pool.mana.clear();
    t.g.untap(isle);
    t.battlefield(P0, "Swamp");
    for (i, m) in [(0, ManaType::U), (1, ManaType::B)] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
        act(&mut t, P0, isle, "add {u} or {b}", &[]).unwrap();
        assert_eq!(t.stack_len(), 0, "a mana ability doesn't use the stack");
        assert!(t.obj_now(isle).tapped);
        assert_eq!(pool(&t, P0, m), 1);
        assert_eq!(pool_size(&t, P0), 1);
        t.g.player_mut(P0).mana_pool.mana.clear();
        t.g.untap(isle);
    }
    // "A Swamp" is the land type: a nonbasic Swamp counts.
    let mut t = TestGame::new(2);
    let isle = t.battlefield(P0, "Tainted Isle");
    t.battlefield(P0, "Bayou");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    act(&mut t, P0, isle, "add {u} or {b}", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::B), 1);
}

#[test]
fn sliver_hive_mana_only_casts_sliver_spells() {
    cr!("605.1a", "106.6");
    // "{T}: Add one mana of any color. Spend this mana only to cast a Sliver spell."
    supported("Sliver Hive");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Sliver Hive");
    let f = t.battlefield(P0, "Forest");
    // W, U, B, R, G: index 4 is green.
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    act(&mut t, P0, h, "sliver spell", &[]).unwrap();
    assert!(t.obj_now(h).tapped);
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(
        t.cast(P0, bears).try_go().is_err(),
        "the Hive's mana can't pay for a non-Sliver"
    );
    assert_eq!(pool(&t, P0, ManaType::G), 1, "still unspent");
    assert!(!t.obj_now(f).tapped);
    let sliver = t.hand(P0, "Muscle Sliver");
    t.cast(P0, sliver).go();
    assert_eq!(pool_size(&t, P0), 0, "spent on the Sliver spell");
    assert!(t.obj_now(f).tapped);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Muscle Sliver").len(), 1);
}

#[test]
fn llanowar_tribe_adds_three_green() {
    cr!("605.1a", "605.3b", "602.5a");
    // "{T}: Add {G}{G}{G}."
    supported("Llanowar Tribe");
    let mut t = TestGame::new(2);
    let sick = t.battlefield_sick(P0, "Llanowar Tribe");
    assert!(
        act(&mut t, P0, sick, "add", &[]).is_err(),
        "summoning sick: can't {{T}}"
    );
    let tr = t.battlefield(P0, "Llanowar Tribe");
    act(&mut t, P0, tr, "add", &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert!(t.obj_now(tr).tapped);
    assert_eq!(pool(&t, P0, ManaType::G), 3);
    assert_eq!(pool_size(&t, P0), 3);
    assert_eq!(pool_size(&t, P1), 0);
}
