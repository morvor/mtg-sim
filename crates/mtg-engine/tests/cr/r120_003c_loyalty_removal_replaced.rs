//! CR 120.3c, 614.1: a replacement effect that changes how many loyalty counters damage
//! removes from a planeswalker (Deification's rule, the `loyalty_removed_by_damage` hook):
//! "if damage dealt to a planeswalker you control of the chosen type would result in all
//! loyalty counters on it being removed, instead all but one of those counters are
//! removed." Tested with a custom permanent carrying Deification's marker (its type
//! choice set directly) until the card itself is supported.

use super::r600_common::CB;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const LAST_LOYALTY: &str = mtg_engine::cards::deification::LAST_LOYALTY;

/// A permanent with Deification's last ability, "Jace" as its chosen type.
fn shrine(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = CB::new("Shrine")
        .enchantment()
        .ability(mtg_engine::cards::marker_if(
            LAST_LOYALTY,
            Condition::Exists(Filter::creature().you_control()),
            "As long as you control a creature, ...",
        ))
        .build();
    let id = t.custom(p, def, Zone::Battlefield);
    t.g.objects[id.0 as usize].choices.creature_type = Some("Jace".into());
    id
}

/// P1 casts Lightning Bolt (3 damage) at `pw`.
fn bolt(t: &mut TestGame, pw: ObjectId) {
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).target(pw).go();
    t.resolve_all();
}

#[test]
fn damage_that_would_remove_all_loyalty_counters_leaves_one() {
    cr!("120.3c", "614.1");
    let mut t = TestGame::new(2);
    shrine(&mut t, P0);
    t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    bolt(&mut t, jace);
    assert!(t.on_battlefield(jace));
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    // The damage was still dealt.
    assert!(t.g.history.objects_dealt_damage.contains(&jace));
}

#[test]
fn not_without_a_creature_or_for_another_type_or_for_partial_damage() {
    cr!("120.3c", "614.1");
    // No creature.
    let mut t = TestGame::new(2);
    shrine(&mut t, P0);
    let jace = t.battlefield(P0, "Jace Beleren");
    bolt(&mut t, jace);
    assert!(!t.on_battlefield(jace));
    // Another planeswalker type.
    let mut t = TestGame::new(2);
    shrine(&mut t, P0);
    t.battlefield(P0, "Grizzly Bears");
    let lili = t.battlefield(P0, "Liliana of the Veil");
    bolt(&mut t, lili);
    assert!(!t.on_battlefield(lili));
    // Damage that leaves some counters anyway removes as many as usual.
    let mut t = TestGame::new(2);
    shrine(&mut t, P0);
    t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.g.objects[jace.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 4);
    bolt(&mut t, jace);
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
}
