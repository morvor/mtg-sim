//! Suffocation (hand-written, `src/cards/suffocation.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn burn(t: &mut TestGame, p: PlayerId, name: &str, target: PlayerId) {
    t.lands(p, "Mountain", 1);
    let c = t.hand(p, name);
    t.cast_with(p, c, &[Entity::Player(target)]).unwrap();
    t.resolve_all();
}

#[test]
fn damages_the_controller_of_the_last_red_spell_that_dealt_damage_to_you() {
    cr!("120.3a");
    let mut t = TestGame::new(3);
    burn(&mut t, P1, "Shock", P0);
    burn(&mut t, P2, "Lightning Bolt", P0);
    assert_eq!(t.life(P0), 15);
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Suffocation");
    t.cast_with(P0, s, &[]).unwrap();
    t.resolve_all();
    // P2's Lightning Bolt was the last one.
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn red_spells_that_damaged_another_player_dont_count() {
    cr!("120.3a");
    let mut t = TestGame::new(3);
    burn(&mut t, P1, "Shock", P0);
    burn(&mut t, P2, "Lightning Bolt", P1);
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Suffocation");
    t.cast_with(P0, s, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 4);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn it_cant_be_cast_without_such_damage() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Suffocation");
    assert!(t.cast_with(P0, s, &[]).is_err());
}
