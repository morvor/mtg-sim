//! Rulings batch P220 — pack tactics: Tiger-Tribe Hunter ("Whenever this creature
//! attacks, if you attacked with creatures with total power 6 or greater this combat, you
//! may sacrifice another creature. When you do, this creature deals damage equal to the
//! sacrificed creature's power to target creature.").

use crate::r_p130_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0's Tiger-Tribe Hunter (4/4) attacks with `others`; P1 has a Colossal Dreadmaw (6/6).
/// Returns (hunter, dreadmaw).
fn hunter_attacks(t: &mut TestGame, others: &[ObjectId]) -> (ObjectId, ObjectId) {
    supported("Tiger-Tribe Hunter");
    let hunter = t.battlefield(P0, "Tiger-Tribe Hunter");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    let attackers: Vec<(ObjectId, Entity)> = std::iter::once(hunter)
        .chain(others.iter().copied())
        .map(|a| (a, Entity::Player(P1)))
        .collect();
    attack_with(t, &attackers);
    (hunter, dreadmaw)
}

#[test]
fn pack_tactics_needs_total_power_six_among_the_attackers() {
    cr!("603.4", "508.1");
    // Alone (power 4): no trigger.
    let mut t = TestGame::new(2);
    hunter_attacks(&mut t, &[]);
    assert_eq!(t.stack_len(), 0);
    // With a Grizzly Bears (4 + 2 = 6): it triggers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    hunter_attacks(&mut t, &[bears]);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn tiger_tribe_hunter_targets_only_after_the_sacrifice_using_its_last_known_power() {
    cr!("603.12", "608.2h");
    ruling!(
        "Tiger-Tribe Hunter",
        "The pack tactics ability goes on the stack without a target. If you sacrifice another creature as it resolves, the reflexive triggered ability will trigger and choose a target for it at that time."
    );
    ruling!(
        "Tiger-Tribe Hunter",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how much damage is dealt."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    // The Giant is a 5/5 with two +1/+1 counters.
    t.g.add_counters(Entity::Object(giant), counters::PLUS1, 2, None);
    t.g.recompute();
    let from = t.asked().len();
    let (_, dreadmaw) = hunter_attacks(&mut t, &[giant]);
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(dreadmaw)]);
    resolve_into_reflexive(&mut t, &[Entity::Object(dreadmaw)]);
    assert!(!t.on_battlefield(giant));
    t.resolve();
    // The Giant's power as it last existed: 5.
    assert_eq!(t.obj(dreadmaw).damage, 5);
}
