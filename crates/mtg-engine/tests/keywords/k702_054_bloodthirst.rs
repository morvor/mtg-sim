//! CR 702.54 Bloodthirst.

use crate::common_k702_011_017::{assert_supported, custom_card, give_mana_for};
use crate::common_k702_052_066::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PLUS1: &str = "+1/+1";

/// Deals `n` damage to `to` from a source `controller` controls (a Shock-like effect).
fn deal_damage(t: &mut TestGame, controller: PlayerId, n: i32, to: Entity) {
    let src = t.battlefield(controller, "Prodigal Pyromancer");
    run_effect(
        t,
        Some(src),
        controller,
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(n),
            to: Sel::Target(0),
        },
        &[to],
    );
}

#[test]
fn bloodthirst_adds_counters_only_if_an_opponent_was_dealt_damage_this_turn() {
    cr!("702.54", "702.54a");
    assert_supported("Gorehorn Minotaurs");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Gorehorn Minotaurs");
    let first = t.hand(P0, "Gorehorn Minotaurs");
    t.cast(P0, first).go();
    t.resolve();
    assert_eq!(t.counters(first, PLUS1), 0);
    assert_eq!(t.pt(first), (3, 3));
    // Damage to an opponent this turn.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    give_mana_for(&mut t, P0, "Gorehorn Minotaurs");
    let second = t.hand(P0, "Gorehorn Minotaurs");
    t.cast(P0, second).go();
    t.resolve();
    assert_eq!(t.counters(second, PLUS1), 2);
    assert_eq!(t.pt(second), (5, 5));
    // It applies however the permanent enters, cast or not.
    let third = t.enter(P0, "Gorehorn Minotaurs");
    assert_eq!(t.counters(third, PLUS1), 2);
}

#[test]
fn only_damage_to_an_opponent_this_turn_counts() {
    cr!("702.54a");
    let mut t = TestGame::new(2);
    // Damage to yourself, or to an opponent's creature, doesn't count.
    deal_damage(&mut t, P1, 2, Entity::Player(P0));
    let bears = t.battlefield(P1, "Grizzly Bears");
    deal_damage(&mut t, P0, 1, Entity::Object(bears));
    let prowler = t.enter(P0, "Bloodscale Prowler");
    assert_eq!(t.counters(prowler, PLUS1), 0);
    // Damage dealt to the opponent by one of their own sources counts.
    deal_damage(&mut t, P1, 1, Entity::Player(P1));
    let prowler = t.enter(P0, "Bloodscale Prowler");
    assert_eq!(t.counters(prowler, PLUS1), 1);
    // Damage dealt last turn doesn't.
    t.advance_to(P1, Step::PrecombatMain);
    let other = t.enter(P0, "Bloodscale Prowler");
    assert_eq!(t.counters(other, PLUS1), 0);
}

#[test]
fn combat_damage_to_an_opponent_turns_on_bloodthirst() {
    cr!("702.54a");
    assert_supported("Carnage Wurm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    t.advance_to(P0, Step::PostcombatMain);
    give_mana_for(&mut t, P0, "Carnage Wurm");
    let wurm = t.hand(P0, "Carnage Wurm");
    t.cast(P0, wurm).go();
    t.resolve();
    assert_eq!(t.pt(wurm), (9, 9));
}

#[test]
fn bloodthirst_x_counts_all_damage_dealt_to_your_opponents_this_turn() {
    cr!("702.54b");
    ruling!(
        "Indoraptor, the Perfect Hybrid",
        "counts all damage dealt to your opponents this turn, not just damage dealt by sources you control"
    );
    let mut t = TestGame::new(3);
    let raptor = t.enter(P0, "Indoraptor, the Perfect Hybrid");
    assert_eq!(t.counters(raptor, PLUS1), 0);
    // Damage to two opponents, from sources of any controller, is added up; damage to
    // you isn't.
    deal_damage(&mut t, P0, 3, Entity::Player(P1));
    deal_damage(&mut t, P2, 2, Entity::Player(P2));
    deal_damage(&mut t, P1, 4, Entity::Player(P0));
    let raptor = t.enter(P0, "Indoraptor, the Perfect Hybrid");
    assert_eq!(t.counters(raptor, PLUS1), 5);
    assert_supported("Petrified Wood-Kin");
    let kin = t.enter(P0, "Petrified Wood-Kin");
    assert_eq!(t.pt(kin), (8, 8));
}

#[test]
fn each_instance_of_bloodthirst_applies_separately() {
    cr!("702.54c");
    ruling!("Bloodlord of Vaasgoth", "Multiple instances of bloodthirst are cumulative");
    assert_supported("Bloodlord of Vaasgoth");
    assert_supported("Vampire Outcasts");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodlord of Vaasgoth");
    deal_damage(&mut t, P0, 1, Entity::Player(P1));
    // "Whenever you cast a Vampire creature spell, it gains bloodthirst 3": with its own
    // bloodthirst 2, it enters with five counters.
    give_mana_for(&mut t, P0, "Vampire Outcasts");
    let outcasts = t.hand(P0, "Vampire Outcasts");
    t.cast(P0, outcasts).go();
    t.resolve_all();
    assert!(t.on_battlefield(outcasts));
    assert_eq!(t.counters(outcasts, PLUS1), 5);
    // A custom creature with two printed instances.
    let def = custom_card(
        "Doubly Thirsty Vampire",
        "Creature — Vampire",
        Some((1, 1)),
        "Bloodthirst 1\nBloodthirst 2",
    );
    let vamp = enter_def(&mut t, P0, def);
    assert_eq!(t.counters(vamp, PLUS1), 3);
}

#[test]
fn bloodthirst_granted_on_casting_needs_the_spell_to_be_cast() {
    cr!("702.54a");
    ruling!(
        "Bloodlord of Vaasgoth",
        "Vampire creatures you put directly onto the battlefield without casting them"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodlord of Vaasgoth");
    deal_damage(&mut t, P0, 1, Entity::Player(P1));
    let outcasts = t.enter(P0, "Vampire Outcasts");
    assert_eq!(t.counters(outcasts, PLUS1), 2);
}
