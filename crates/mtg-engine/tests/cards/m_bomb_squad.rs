//! Bomb Squad (hand-written, `src/cards/bomb_squad.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_creature_with_four_fuse_counters_is_destroyed_and_deals_4_to_its_controller() {
    cr!("603.8");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bomb Squad");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(giant), "fuse", 4, None);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn three_counters_do_nothing() {
    cr!("603.8");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bomb Squad");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(giant), "fuse", 3, None);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn each_bomb_squad_triggers() {
    cr!("603.8");
    ruling!("Bomb Squad", "both Bomb Squad abilities will trigger causing 4 damage each");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bomb Squad");
    t.battlefield(P0, "Bomb Squad");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(giant), "fuse", 4, None);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn regeneration_doesnt_stop_the_damage() {
    cr!("701.19a");
    ruling!("Bomb Squad", "the fuse counters are still removed and the four damage is still dealt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bomb Squad");
    let troll = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.g.add_counters(Entity::Object(troll), "fuse", 4, None);
    t.settle();
    // P1 regenerates the Boa in response.
    t.activate(P1, troll, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(troll));
    assert_eq!(t.counters(troll, "fuse"), 0);
    assert_eq!(t.life(P1), 16);
}
