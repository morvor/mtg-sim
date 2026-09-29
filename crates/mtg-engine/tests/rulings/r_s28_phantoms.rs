//! Rulings batch S28 — the Phantom creatures ("If damage would be dealt to this creature,
//! prevent that damage. Remove a +1/+1 counter from this creature."): a prevention effect
//! with an additional instruction (CR 615.5). It prevents damage even without counters, is
//! applied once to simultaneous damage from several sources, and still removes a counter
//! when the damage can't be prevented (CR 615.12).

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::damage;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PLUS1: &str = "+1/+1";

#[test]
fn unpreventable_damage_is_dealt_and_a_counter_is_still_removed() {
    cr!("615.12", "615.5");
    ruling!(
        "Phantom Tiger",
        "If damage that would be dealt to this creature can't be prevented, the damage is dealt and a +1/+1 counter is removed from this creature."
    );
    supported("Phantom Tiger");
    // A 1/0 that enters with two +1/+1 counters.
    let mut t = TestGame::new(2);
    let tiger = t.enter(P0, "Phantom Tiger");
    assert_eq!(t.pt(tiger), (3, 2));
    let giant = t.battlefield(P1, "Hill Giant");
    // Prevented: a counter is removed.
    damage(&mut t, giant, 2, tiger);
    assert_eq!(t.obj_now(tiger).damage, 0);
    assert_eq!(t.counters(tiger, PLUS1), 1);
    // Leyline of Punishment: "Damage can't be prevented." 1 damage is dealt and the last
    // counter removed, so the 1/0 with 1 damage dies.
    t.battlefield(P1, "Leyline of Punishment");
    damage(&mut t, giant, 1, tiger);
    assert!(t.in_graveyard(P0, "Phantom Tiger"));
}

#[test]
fn damage_from_several_sources_at_once_removes_one_counter() {
    cr!("615.5", "510.2");
    ruling!(
        "Phantom Tiger",
        "If this creature would be dealt damage from multiple sources at the same time (say, because it's blocked by multiple creatures), all of the damage is prevented and only one +1/+1 counter is removed."
    );
    let mut t = TestGame::new(2);
    let tiger = t.enter(P0, "Phantom Tiger");
    t.g.objects[tiger.0 as usize].summoning_sick = false;
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(tiger, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(a, tiger), (b, tiger)]);
    assert!(t.on_battlefield(tiger));
    assert_eq!(t.obj_now(tiger).damage, 0);
    assert_eq!(t.counters(tiger, PLUS1), 1);
}

#[test]
fn damage_is_prevented_even_without_counters() {
    cr!("615.5", "615.1a");
    ruling!(
        "Phantom Tiger",
        "If this creature has no +1/+1 counters on it (but is still on the battlefield because something else is raising its toughness), damage that would be dealt to it is still prevented."
    );
    let mut t = TestGame::new(2);
    let tiger = t.enter(P0, "Phantom Tiger");
    t.battlefield(P0, "Glorious Anthem");
    t.g.remove_counters(Entity::Object(tiger), PLUS1, 2);
    t.g.recompute();
    assert_eq!(t.pt(tiger), (2, 1));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 3, tiger);
    assert!(t.on_battlefield(tiger));
    assert_eq!(t.obj_now(tiger).damage, 0);
    assert_eq!(t.counters(tiger, PLUS1), 0);
}
