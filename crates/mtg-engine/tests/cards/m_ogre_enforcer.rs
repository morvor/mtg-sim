//! Ogre Enforcer (hand-written, `src/cards/ogre_enforcer.rs`): it isn't destroyed by
//! lethal damage unless lethal damage from a single source is marked on it (an exception
//! to CR 704.5g).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn damage_from_several_sources_doesnt_add_up() {
    cr!("704.5g");
    ruling!("Ogre Enforcer", "not possible to team up sources");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Ogre Enforcer");
    t.lands(P0, "Mountain", 2);
    for _ in 0..2 {
        let b = t.hand(P0, "Lightning Bolt");
        t.cast(P0, b).target(ogre).go();
        t.resolve();
    }
    assert_eq!(t.obj_now(ogre).damage, 6);
    assert!(t.on_battlefield(ogre));
}

#[test]
fn blocked_by_two_creatures_it_survives() {
    cr!("704.5g", "510.2");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Ogre Enforcer");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ogre, Entity::Player(P1))], &[(b1, ogre), (b2, ogre)]);
    assert!(t.on_battlefield(ogre));
}

#[test]
fn lethal_damage_from_one_source_destroys_it() {
    cr!("704.5g");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Ogre Enforcer");
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(ogre).go();
    t.resolve();
    t.lands(P0, "Mountain", 1);
    let f = t.hand(P0, "Flame Slash");
    t.cast(P0, f).target(ogre).go();
    t.resolve();
    assert!(!t.on_battlefield(ogre));
}

#[test]
fn toughness_zero_still_dies() {
    cr!("704.5f");
    ruling!("Ogre Enforcer", "If its toughness falls to zero or less");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Ogre Enforcer");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Grasp of Darkness");
    t.cast(P0, s).target(ogre).go();
    t.resolve();
    assert!(!t.on_battlefield(ogre));
}
