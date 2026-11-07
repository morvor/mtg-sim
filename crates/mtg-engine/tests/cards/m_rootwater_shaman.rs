//! Rootwater Shaman (hand-written, `src/cards/rootwater_shaman.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn auras_with_enchant_creature_have_flash() {
    cr!("702.5a", "702.8a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rootwater Shaman");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Plains", 2);
    let pac = t.hand(P0, "Pacifism");
    t.cast(P0, pac).target(bear).go();
    t.resolve();
    assert_eq!(t.obj_now(pac).attached_to, Some(Entity::Object(bear)));
}

#[test]
fn other_auras_and_spells_dont() {
    cr!("702.5a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rootwater Shaman");
    let forest = t.battlefield(P0, "Forest");
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Forest", 2);
    // Wild Growth has enchant land.
    let wg = t.hand(P0, "Wild Growth");
    assert!(t.cast(P0, wg).target(forest).try_go().is_err());
    let bear = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bear).try_go().is_err());
    // Without the Shaman, Pacifism can't be cast now.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Plains", 2);
    let pac = t.hand(P0, "Pacifism");
    assert!(t.cast(P0, pac).target(bear).try_go().is_err());
}
