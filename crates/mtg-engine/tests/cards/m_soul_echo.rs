//! Soul Echo (hand-written, `src/cards/soul_echo.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn to_upkeep(t: &mut TestGame, opponent_agrees: bool) -> ObjectId {
    let echo = t.battlefield(P0, "Soul Echo");
    t.g.add_counters(Entity::Object(echo), "echo", 3, None);
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, opponent_agrees);
    t.advance_to(P0, Step::Draw);
    echo
}

#[test]
fn damage_removes_echo_counters_instead_once_the_opponent_chooses() {
    cr!("614.1a");
    ruling!("Soul Echo", "the effect still replaces the damage and uselessly tries to remove counters");
    let mut t = TestGame::new(2);
    let echo = to_upkeep(&mut t, true);
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(echo, "echo"), 1);
    // Out of counters: the damage is still replaced.
    t.g.deal_damage(src, Entity::Player(P0), 4, false);
    t.settle();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(echo, "echo"), 0);
    assert!(t.on_battlefield(echo));
}

#[test]
fn damage_is_dealt_normally_if_the_opponent_declines() {
    cr!("614.1a");
    let mut t = TestGame::new(2);
    let echo = to_upkeep(&mut t, false);
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(echo, "echo"), 3);
}

#[test]
fn sacrificed_without_echo_counters() {
    cr!("701.21a");
    let mut t = TestGame::new(2);
    let echo = t.battlefield(P0, "Soul Echo");
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Draw);
    assert!(!t.on_battlefield(echo));
}
