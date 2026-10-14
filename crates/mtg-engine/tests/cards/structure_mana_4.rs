//! Structure-coverage tests (round 4): mana-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{act, next_turn, untapped};
use super::structure_seed_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId, m: ManaType) -> u32 {
    t.g.player(p).mana_pool.count(m) as u32
}

fn pool_size(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.mana.len()
}

#[test]
fn gathering_place_adds_two_colors_only_with_a_basic_or_this_turn() {
    cr!("605.1a", "605.3b", "602.1b");
    // "{T}: Add {G} or {W}. Activate only if this land entered this turn or if you control
    // a basic land."
    supported("Gathering Place");
    let mut t = TestGame::new(2);
    let g = t.enter(P0, "Gathering Place");
    t.battlefield(P0, "Taiga"); // not basic
    t.battlefield(P1, "Forest"); // not yours
    // It entered this turn: no basic land needed. Index 0 is green.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    act(&mut t, P0, g, "add {g} or {w}", &[]).unwrap();
    assert_eq!(t.stack_len(), 0, "a mana ability doesn't use the stack");
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    assert_eq!(pool_size(&t, P0), 1);
    assert!(t.obj_now(g).tapped);
    // The next turn it didn't enter this turn, and P0 controls no basic land.
    next_turn(&mut t);
    t.g.untap(g);
    assert!(
        act(&mut t, P0, g, "add {g} or {w}", &[]).is_err(),
        "neither condition holds"
    );
    t.clear_answers();
    t.battlefield(P0, "Plains");
    // Index 1 is white.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    act(&mut t, P0, g, "add {g} or {w}", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::W), 1);
    assert_eq!(pool_size(&t, P0), 1);
}

#[test]
fn depletion_land_removes_a_counter_for_two_mana_then_is_sacrificed() {
    cr!("605.1a", "118.3", "122.1");
    // Hickory Woodlot: "{T}, Remove a depletion counter from this land: Add {G}{G}. If
    // there are no depletion counters on this land, sacrifice it."
    supported("Hickory Woodlot");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hickory Woodlot");
    assert!(act(&mut t, P0, h, "add", &[]).is_err(), "no depletion counter");
    t.g.add_counters(Entity::Object(h), "depletion", 2, None);
    act(&mut t, P0, h, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::G), 2);
    assert_eq!(pool_size(&t, P0), 2);
    assert_eq!(t.counters(h, "depletion"), 1);
    assert!(t.on_battlefield(h), "a counter is left");
    t.g.untap(h);
    act(&mut t, P0, h, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::G), 4);
    assert!(!t.on_battlefield(h), "no counters left: sacrificed");
    assert!(t.in_graveyard(P0, "Hickory Woodlot"));
}

#[test]
fn koskun_keep_filters_two_mana_into_one_of_two_colors() {
    cr!("605.1a", "118.3");
    // "{2}, {T}: Add {B} or {G}."
    supported("Koskun Keep");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Koskun Keep");
    t.battlefield(P0, "Wastes");
    assert!(
        act(&mut t, P0, k, "{2}, {t}", &[]).is_err(),
        "only one mana for {{2}}"
    );
    t.battlefield(P0, "Wastes");
    // Index 1 is green.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    act(&mut t, P0, k, "{2}, {t}", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    assert!(t.obj_now(k).tapped);
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    assert_eq!(pool_size(&t, P0), 1);
}

#[test]
fn devotee_filters_into_three_colors_once_each_turn() {
    cr!("605.1a", "602.1b");
    // Mardu Devotee: "{1}: Add {R}, {W}, or {B}. Activate only once each turn."
    supported("Mardu Devotee");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Mardu Devotee");
    t.lands(P0, "Wastes", 2);
    // Index 2 is black.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    act(&mut t, P0, d, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::B), 1);
    assert_eq!(pool_size(&t, P0), 1);
    assert_eq!(untapped(&t, P0, "Wastes"), 1, "{{1}} was paid");
    assert!(!t.obj_now(d).tapped);
    assert!(act(&mut t, P0, d, "add", &[]).is_err(), "only once each turn");
    t.g.player_mut(P0).mana_pool.mana.clear();
    t.advance_to(P1, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    act(&mut t, P0, d, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::R), 1);
}

#[test]
fn village_mana_only_casts_creature_spells() {
    cr!("605.1a", "106.6");
    // Oakhollow Village: "{T}: Add {G}. Spend this mana only to cast a creature spell."
    supported("Oakhollow Village");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Oakhollow Village");
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, v, "creature spell", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    let gg = t.hand(P0, "Giant Growth");
    assert!(
        t.cast(P0, gg).target(bears).try_go().is_err(),
        "not for an instant"
    );
    let elves = t.hand(P0, "Llanowar Elves");
    assert_eq!(pool(&t, P0, ManaType::G), 1, "still unspent");
    t.cast(P0, elves).go();
    assert_eq!(pool_size(&t, P0), 0, "spent on the creature spell");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn veldt_adds_mana_and_gets_a_depletion_counter() {
    cr!("605.1a", "122.1");
    // "{T}: Add {G} or {W}. Put a depletion counter on this land."
    supported("Veldt");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Veldt");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    act(&mut t, P0, v, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    assert_eq!(pool_size(&t, P0), 1);
    assert_eq!(t.counters(v, "depletion"), 1);
    assert!(t.obj_now(v).tapped);
}

#[test]
fn catalyst_elemental_sacrifices_for_two_red() {
    cr!("605.1a", "701.21a");
    // "Sacrifice this creature: Add {R}{R}."
    supported("Catalyst Elemental");
    let mut t = TestGame::new(2);
    let c = t.battlefield_sick(P0, "Catalyst Elemental");
    act(&mut t, P0, c, "add", &[]).unwrap();
    assert!(!t.on_battlefield(c), "no {{T}}: summoning sickness doesn't matter");
    assert!(t.in_graveyard(P0, "Catalyst Elemental"));
    assert_eq!(pool(&t, P0, ManaType::R), 2);
    assert_eq!(pool_size(&t, P0), 2);
}

#[test]
fn golden_egg_sacrifices_for_one_mana_of_any_color() {
    cr!("605.1a", "118.3", "701.21a");
    // "{1}, {T}, Sacrifice this artifact: Add one mana of any color."
    supported("Golden Egg");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Golden Egg");
    assert!(act(&mut t, P0, e, "add", &[]).is_err(), "no mana for {{1}}");
    let w = t.battlefield(P0, "Wastes");
    // W, U, B, R, G: index 1 is blue.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    act(&mut t, P0, e, "add", &[]).unwrap();
    assert!(t.obj_now(w).tapped);
    assert!(!t.on_battlefield(e));
    assert_eq!(pool(&t, P0, ManaType::U), 1);
    assert_eq!(pool_size(&t, P0), 1);
}
