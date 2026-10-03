//! Wall of Shadows (hand-written, `src/cards/wall_of_shadows.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_spell_that_can_target_only_walls_cant_target_it() {
    cr!("115.4");
    let mut t = TestGame::new(2);
    let ws = t.battlefield(P1, "Wall of Shadows");
    let stone = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 2);
    let tunnel = t.hand(P0, "Tunnel");
    // Wall of Shadows isn't a legal target: the only one is the other Wall.
    let spell = t.cast(P0, tunnel).target(ws).go();
    let si = t.g.obj(spell).stack.clone().unwrap();
    assert_eq!(si.chosen[0].targets[0], vec![Entity::Object(stone)]);
    t.resolve();
    assert!(!t.on_battlefield(stone));
    assert!(t.on_battlefield(ws));
}

#[test]
fn an_ability_that_can_target_only_walls_cant_target_it() {
    cr!("115.4");
    let mut t = TestGame::new(2);
    let ws = t.battlefield(P1, "Wall of Shadows");
    let team = t.battlefield(P0, "Dwarven Demolition Team");
    // No legal target: the ability can't be activated.
    let _ = t.activate(P0, team, 0, &[ws.into()]);
    t.resolve_all();
    assert!(t.on_battlefield(ws));
}

#[test]
fn a_modal_spell_with_another_mode_can_target_it() {
    cr!("115.4", "700.2");
    ruling!("Wall of Shadows", "Can be targeted by a modal spell that can target a non-Wall in one of its modes");
    let mut t = TestGame::new(2);
    let ws = t.battlefield(P1, "Wall of Shadows");
    t.lands(P0, "Mountain", 1);
    let charm = t.hand(P0, "Chaos Charm");
    t.cast(P0, charm).modes(&[0]).target(ws).go();
    t.resolve();
    assert!(!t.on_battlefield(ws));
}
