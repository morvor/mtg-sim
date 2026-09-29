//! Rulings batch S29 — excess damage (CR 120.10) with Bottle-Cap Blast: "Bottle-Cap Blast
//! deals 5 damage to any target. If excess damage was dealt to a permanent this way,
//! create that many tapped Treasure tokens." Excess damage is damage beyond lethal damage
//! for a creature (counting damage already marked), beyond its loyalty for a
//! planeswalker, and beyond its defense for a battle.

use crate::r_s01_common::{supported, with_subtype};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Bottle-Cap Blast at `target`; returns how many Treasures P0 then controls
/// (all tapped).
fn blast(t: &mut TestGame, target: Entity) -> usize {
    supported("Bottle-Cap Blast");
    cast_new(t, P0, "Bottle-Cap Blast", &[target]);
    t.resolve_all();
    let treasures = with_subtype(t, P0, "Treasure");
    assert!(treasures.iter().all(|x| t.obj_now(*x).tapped));
    treasures.len()
}

#[test]
fn excess_damage_to_a_planeswalker_or_battle_is_beyond_its_loyalty_or_defense() {
    cr!("120.10", "306.8", "310.6");
    ruling!(
        "Bottle-Cap Blast",
        "For a planeswalker, excess damage means damage in excess of the planeswalker's loyalty (the number of loyalty counters it has on it). For a battle, excess damage means damage in excess of the battle's defense (the number of defense counters it has on it)."
    );
    // Jace Beleren with three loyalty counters: 2 excess.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    assert_eq!(blast(&mut t, Entity::Object(jace)), 2);
    // With five loyalty counters: none.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    put_counters(&mut t, jace, counters::LOYALTY, 2);
    assert_eq!(blast(&mut t, Entity::Object(jace)), 0);
    // A battle with four defense counters (Invasion of Segovia): 1 excess.
    let mut t = TestGame::new(2);
    let battle = t.battlefield(P1, "Invasion of Segovia // Caetus, Sea Tyrant of Segovia");
    assert_eq!(t.counters(battle, counters::DEFENSE), 4);
    assert_eq!(blast(&mut t, Entity::Object(battle)), 1);
    // A player can't be dealt excess damage.
    let mut t = TestGame::new(2);
    assert_eq!(blast(&mut t, Entity::Player(P1)), 0);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn excess_damage_to_a_creature_counts_damage_already_marked() {
    cr!("120.10", "120.6");
    ruling!(
        "Bottle-Cap Blast",
        "Excess damage has been dealt to a creature if the damage dealt to it is greater than lethal damage. Usually, this means damage greater than its toughness, although damage already marked on the creature is taken into account."
    );
    // A 2/2: 3 excess.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(blast(&mut t, Entity::Object(bears)), 3);
    // A 3/3 Hill Giant with 2 damage already marked: lethal damage is 1, so 4 excess.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_and_resolve(&mut t, P0, "Shock", &[Entity::Object(giant)]);
    assert_eq!(damage_marked(&t, giant), 2);
    assert_eq!(blast(&mut t, Entity::Object(giant)), 4);
    // A 6/6 isn't dealt excess damage.
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    assert_eq!(blast(&mut t, Entity::Object(dreadmaw)), 0);
}
