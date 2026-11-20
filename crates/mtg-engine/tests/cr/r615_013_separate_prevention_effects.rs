//! Separate prevention effects applied to simultaneous damage stay separate: the effect
//! each creature's shield counters create removes a counter from that creature (CR 122.1c,
//! 615.5), and two protection abilities of one player that each prevent one creature's
//! combat damage are two applications, each triggering "whenever damage that would be
//! dealt to you is prevented" (CR 615.13). The engine tells the effects apart by a number
//! made from each effect's key (its permanent or player and which of its effects it is), so
//! a number that lost the permanent or the ability would merge them.

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn pyroclasm_removes_a_shield_counter_from_each_shielded_creature() {
    cr!("122.1c", "615.5");
    ruling!(
        "Swooping Protector",
        "If a permanent that would be dealt damage has more than one shield counter on it, that damage is prevented and only one shield counter is removed."
    );
    supported("Swooping Protector");
    // Swooping Protector: "This creature enters with a shield counter on it."
    let mut t = TestGame::new(2);
    let first = t.enter(P0, "Swooping Protector");
    let second = t.enter(P0, "Swooping Protector");
    t.resolve_all();
    // Boon of Safety: "Put a shield counter on target creature. Scry 1."
    cast_resolve(&mut t, P0, "Boon of Safety", &[obj(first)]);
    assert_eq!(t.counters(first, "shield"), 2);
    assert_eq!(t.counters(second, "shield"), 1);
    // "Pyroclasm deals 2 damage to each creature.": each creature's own effect prevents its
    // damage and removes one of its counters.
    cast_resolve(&mut t, P0, "Pyroclasm", &[]);
    for c in [first, second] {
        assert!(t.on_battlefield(c));
        assert_eq!(damage_on(&t, c), 0);
    }
    assert_eq!(t.counters(first, "shield"), 1);
    assert_eq!(t.counters(second, "shield"), 0);
}

#[test]
fn two_runed_halos_preventing_two_attackers_damage_trigger_selfless_squire_twice() {
    cr!("615.13", "702.16e", "510.2");
    ruling!(
        "Selfless Squire",
        "Selfless Squire's second triggered ability doesn't trigger only when damage is prevented by its first triggered ability's effect."
    );
    supported("Runed Halo");
    supported("Selfless Squire");
    // Selfless Squire: "Whenever damage that would be dealt to you is prevented, put that
    // many +1/+1 counters on this creature." Runed Halo: "You have protection from the
    // chosen card name."
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Selfless Squire");
    for name in ["Grizzly Bears", "Hill Giant"] {
        t.answer(P0, DecisionKind::Name, Answer::Text(name.to_string()));
        t.enter(P0, "Runed Halo");
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (bears, Entity::Player(P0)),
            (giant, Entity::Player(P0)),
        ]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 20);
    // Each Halo's protection prevents one creature's damage: two prevention effects, each
    // applied once, so two triggers (for 2 and for 3), not one for 5.
    assert_eq!(t.stack_len(), 2, "one trigger for each Halo's protection");
    t.resolve_all();
    assert_eq!(plus_counters(&t, squire), 5);
}
