//! CR 120.10, 603.2: "is dealt excess damage" triggers for combat damage too, unless the
//! ability says "noncombat".

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn excess_combat_damage_triggers_an_excess_damage_ability() {
    cr!("120.10", "603.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // "Whenever a creature is dealt excess damage, you gain 1 life."
    let watcher = CB::new("Excess Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::DealtExcessDamage {
                filter: Filter::creature(),
                noncombat_only: false,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P0, watcher, Zone::Battlefield);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // The 3/3 Giant is blocked by a 1/1: 2 excess combat damage.
    t.attack(&[(giant, Entity::Player(P1))], &[(elves, giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.life(P0), 21);
}
