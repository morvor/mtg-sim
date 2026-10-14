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

/// Gnoll Hunter (2/2; pack tactics: put a +1/+1 counter on it) attacks with Hill Giant
/// (3/3) and Grizzly Bears (2/2): total power 7. Returns (gnoll, giant, bears).
fn gnoll_attacks(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    supported("Gnoll Hunter");
    let gnoll = t.battlefield(P0, "Gnoll Hunter");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        t,
        &[
            (gnoll, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    assert_eq!(t.stack_len(), 1, "pack tactics triggers (2 + 3 + 2 = 7)");
    (gnoll, giant, bears)
}

#[test]
fn pack_tactics_counts_the_power_the_attackers_had_as_they_were_declared() {
    cr!("603.4", "508.1");
    // An attacker removed from combat (here, it left the battlefield) and another whose
    // power dropped after attackers were declared (the attacking creatures now have total
    // power 3): you still "attacked with" creatures with total power 7, so the intervening "if" is still true as the ability resolves.
    let mut t = TestGame::new(2);
    let (gnoll, giant, bears) = gnoll_attacks(&mut t);
    t.g.destroy(giant, None);
    t.g.add_counters(Entity::Object(bears), counters::MINUS1, 1, None);
    t.g.recompute();
    assert!(!t.on_battlefield(giant));
    t.resolve();
    assert_eq!(t.obj(gnoll).counters.get(counters::PLUS1).copied(), Some(1));
}

#[test]
fn pack_tactics_ignores_power_gained_after_attackers_were_declared() {
    cr!("603.4", "508.1");
    // Gnoll Hunter (2) and Grizzly Bears (2) attack: total 4, no trigger, even though the
    // Bears get two +1/+1 counters right after attackers are declared.
    supported("Gnoll Hunter");
    let mut t = TestGame::new(2);
    let gnoll = t.battlefield(P0, "Gnoll Hunter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(gnoll, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert_eq!(t.stack_len(), 0);
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 4, None);
    t.g.recompute();
    let v = mtg_engine::kw::pack_tactics::ATTACKED_WITH_TOTAL_POWER;
    let ctx = mtg_engine::eval::Ctx::new(Some(gnoll), P0);
    assert_eq!(mtg_engine::custom::custom_value(&t.g, v, &ctx), 4);
}
