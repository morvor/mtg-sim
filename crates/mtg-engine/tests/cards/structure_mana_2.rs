//! Structure-coverage tests (round 2): mana-ability structures shared by many cards that
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
fn sphere_of_the_suns_removes_a_charge_counter_for_any_color() {
    cr!("605.1a", "605.3a", "118.3", "106.1a");
    // "{T}, Remove a charge counter from this artifact: Add one mana of any color."
    supported("Sphere of the Suns");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sphere of the Suns");
    assert!(act(&mut t, P0, s, "add", &[]).is_err(), "no charge counter");
    t.g.add_counters(Entity::Object(s), "charge", 1, None);
    // Options are W, U, B, R, G: index 2 is black.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    act(&mut t, P0, s, "add", &[]).unwrap();
    assert_eq!(t.counters(s, "charge"), 0);
    assert!(t.obj_now(s).tapped);
    assert_eq!(pool(&t, P0, ManaType::B), 1);
    assert_eq!(pool_size(&t, P0), 1);
}

#[test]
fn grim_monolith_adds_three_colorless() {
    cr!("605.1a", "605.3a", "106.1b");
    // "{T}: Add {C}{C}{C}."
    supported("Grim Monolith");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Grim Monolith");
    act(&mut t, P0, m, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::C), 3);
    assert_eq!(pool_size(&t, P0), 3);
    assert!(t.obj_now(m).tapped);
    assert_eq!(pool_size(&t, P1), 0);
}

#[test]
fn elves_of_deep_shadow_adds_black_and_damages_you() {
    cr!("605.1a", "605.3b", "120.3a");
    // "{T}: Add {B}. This creature deals 1 damage to you."
    supported("Elves of Deep Shadow");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Elves of Deep Shadow");
    act(&mut t, P0, e, "add", &[]).unwrap();
    assert_eq!(t.stack_len(), 0, "a mana ability doesn't use the stack");
    assert_eq!(pool(&t, P0, ManaType::B), 1);
    assert_eq!(pool_size(&t, P0), 1);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn horizon_canopy_pays_life_for_one_of_two_colors() {
    cr!("605.1a", "118.3", "119.4");
    // "{T}, Pay 1 life: Add {G} or {W}."
    supported("Horizon Canopy");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Horizon Canopy");
    act(&mut t, P0, c, "pay 1 life", &[]).unwrap();
    assert_eq!(t.life(P0), 19);
    assert_eq!(
        pool(&t, P0, ManaType::G) + pool(&t, P0, ManaType::W),
        1,
        "one mana, green or white"
    );
    assert_eq!(pool_size(&t, P0), 1);
    assert!(t.obj_now(c).tapped);
}

#[test]
fn blightsoil_druid_pays_life_for_green() {
    cr!("605.1a", "118.3", "119.4");
    // "{T}, Pay 1 life: Add {G}."
    supported("Blightsoil Druid");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Blightsoil Druid");
    act(&mut t, P0, d, "add", &[]).unwrap();
    assert_eq!(t.life(P0), 19);
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    assert_eq!(pool_size(&t, P0), 1);
    // A player with 0 life can't pay life (CR 119.4).
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Blightsoil Druid");
    t.g.players[0].life = 0;
    assert!(act(&mut t, P0, d, "add", &[]).is_err());
}

#[test]
fn havenwood_battleground_sacrifices_for_two_green() {
    cr!("605.1a", "118.3", "701.21a");
    // "{T}, Sacrifice this land: Add {G}{G}."
    supported("Havenwood Battleground");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Havenwood Battleground");
    act(&mut t, P0, h, "sacrifice", &[]).unwrap();
    assert!(!t.on_battlefield(h));
    assert!(t.in_graveyard(P0, "Havenwood Battleground"));
    assert_eq!(pool(&t, P0, ManaType::G), 2);
    assert_eq!(pool_size(&t, P0), 2);
}

#[test]
fn koskun_keep_filters_one_mana_into_red() {
    cr!("605.1a", "605.3a", "118.3");
    // "{1}, {T}: Add {R}."
    supported("Koskun Keep");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Koskun Keep");
    assert!(
        act(&mut t, P0, k, "{1}, {t}: add {r}", &[]).is_err(),
        "no mana for {{1}}"
    );
    let w = t.battlefield(P0, "Wastes");
    act(&mut t, P0, k, "{1}, {t}: add {r}", &[]).unwrap();
    assert!(t.obj_now(w).tapped && t.obj_now(k).tapped);
    assert_eq!(pool(&t, P0, ManaType::R), 1);
    assert_eq!(pool_size(&t, P0), 1, "the {{C}} paid the cost");
}

#[test]
fn nantuko_elder_adds_colorless_and_green() {
    cr!("605.1a", "605.3a");
    // "{T}: Add {C}{G}."
    supported("Nantuko Elder");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Nantuko Elder");
    act(&mut t, P0, e, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::C), 1);
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    assert_eq!(pool_size(&t, P0), 2);
}

#[test]
fn shire_scarecrow_filters_mana_once_each_turn() {
    cr!("605.1a", "602.5b");
    // "{1}: Add one mana of any color. Activate only once each turn."
    supported("Shire Scarecrow");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shire Scarecrow");
    t.lands(P0, "Wastes", 2);
    // Index 3 is red.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    act(&mut t, P0, s, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::R), 1);
    assert_eq!(pool_size(&t, P0), 1);
    assert!(!t.obj_now(s).tapped, "no {{T}} in the cost");
    assert!(
        act(&mut t, P0, s, "add", &[]).is_err(),
        "only once each turn"
    );
    // A new turn: it can be activated again.
    t.g.player_mut(P0).mana_pool.mana.clear();
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    act(&mut t, P0, s, "add", &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::R), 1);
}
