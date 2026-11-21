//! Silhouette (hand-written, `src/cards/silhouette.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn shield(t: &mut TestGame, c: ObjectId) {
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Silhouette");
    t.cast(P0, s).target(c).go();
    t.resolve();
}

#[test]
fn prevents_damage_from_a_spell_that_targets_it() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    shield(&mut t, bears);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn damage_from_untargeted_spells_isnt_prevented() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    shield(&mut t, bears);
    t.lands(P0, "Mountain", 2);
    let p = t.hand(P0, "Pyroclasm");
    t.cast(P0, p).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn combat_damage_isnt_prevented() {
    cr!("615.1a", "510.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    shield(&mut t, bears);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    assert!(!t.on_battlefield(bears));
}
