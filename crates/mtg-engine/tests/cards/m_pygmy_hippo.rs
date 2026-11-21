//! Pygmy Hippo (hand-written, `src/cards/pygmy_hippo.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn drains_the_defending_players_lands_for_your_next_main_phase() {
    cr!("605.3a", "106.4");
    let mut t = TestGame::new(2);
    let hippo = t.battlefield(P0, "Pygmy Hippo");
    let lands = t.lands(P1, "Forest", 3);
    t.answer_yes(P0, true);
    t.attack(&[(hippo, Entity::Player(P1))], &[]);
    // No combat damage; their lands were tapped and the mana lost.
    assert_eq!(t.life(P1), 20);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn deals_damage_normally_if_you_dont() {
    cr!("510.1a");
    let mut t = TestGame::new(2);
    let hippo = t.battlefield(P0, "Pygmy Hippo");
    t.lands(P1, "Forest", 3);
    t.answer_yes(P0, false);
    t.attack(&[(hippo, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}
