//! Rulings batch S29 — the Clockwork creatures (Clockwork Avian: "Flying. This creature
//! enters with four +1/+0 counters on it. At end of combat, if this creature attacked or
//! blocked this combat, remove a +1/+0 counter from it. {X}, {T}: Put up to X +1/+0
//! counters on this creature. This ability can't cause the total number of +1/+0 counters
//! on this creature to be greater than four. Activate only during your upkeep.").

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::can_attack;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PLUS1_0: &str = "+1/+0";

/// A Clockwork creature that entered the battlefield under P0's control (with its
/// counters) before this turn.
fn clockwork(t: &mut TestGame, name: &str) -> ObjectId {
    supported(name);
    let id = t.enter(P0, name);
    t.resolve_all();
    t.g.objects[id.0 as usize].summoning_sick = false;
    id
}

/// Removes `n` +1/+0 counters from the object.
fn remove(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.remove_counters(Entity::Object(id), PLUS1_0, n);
    t.g.recompute();
}

/// P0 activates the Clockwork's "{X}, {T}: Put up to X +1/+0 counters ..." ability with
/// X = `x` in P0's upkeep (it doesn't resolve yet).
fn wind_up(t: &mut TestGame, id: ObjectId, x: i64) {
    t.set_step(P0, Step::Upkeep);
    add_mana(t, P0, ManaType::C, x as u32);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    activate_containing(t, P0, id, "Put up to X").expect("the ability");
}

#[test]
fn counters_over_the_maximum_are_simply_not_added() {
    cr!("608.2h", "122.1");
    ruling!(
        "Clockwork Avian",
        "If the ability to add counters resolves when there are already the maximum number of counters on it, any counters over the maximum are simply not added."
    );
    supported("Clockwork Swarm");
    let mut t = TestGame::new(2);
    let avian = clockwork(&mut t, "Clockwork Avian");
    assert_eq!(t.counters(avian, PLUS1_0), 4);
    remove(&mut t, avian, 3);
    // X = 3 with one counter: it gets three (four in all).
    wind_up(&mut t, avian, 3);
    t.resolve_all();
    assert_eq!(t.counters(avian, PLUS1_0), 4);
    // X = 3 with two counters: only two are put on.
    remove(&mut t, avian, 2);
    t.g.objects[avian.0 as usize].tapped = false;
    wind_up(&mut t, avian, 3);
    t.resolve_all();
    assert_eq!(t.counters(avian, PLUS1_0), 4);
    // Counters put on in response count: X = 2 with two counters, then two more are put on
    // before it resolves: none are added.
    remove(&mut t, avian, 2);
    t.g.objects[avian.0 as usize].tapped = false;
    wind_up(&mut t, avian, 2);
    put_counters(&mut t, avian, PLUS1_0, 2);
    t.resolve_all();
    assert_eq!(t.counters(avian, PLUS1_0), 4);
}

#[test]
fn it_loses_a_counter_even_if_its_damage_was_prevented() {
    cr!("603.4", "511.1", "615.1");
    ruling!(
        "Clockwork Avian",
        "Loses a counter even if it is affected by a Fog-like effect which prevents it from dealing damage."
    );
    supported("Fog");
    let mut t = TestGame::new(2);
    let avian = clockwork(&mut t, "Clockwork Avian");
    // Fog: "Prevent all combat damage that would be dealt this turn."
    cast_new(&mut t, P1, "Fog", &[]);
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(avian, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert!(t.g.is_attacking(avian));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(avian, PLUS1_0), 3);
    // A blocking one (P1's Clockwork Swarm) loses one too.
    let mut t = TestGame::new(2);
    let swarm = t.enter(P1, "Clockwork Swarm");
    t.resolve_all();
    let n = t.counters(swarm, PLUS1_0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(swarm, bears)]);
    t.resolve_all();
    assert_eq!(t.counters(swarm, PLUS1_0), n - 1);
    // A Clockwork creature that neither attacked nor blocked keeps its counters.
    let mut t = TestGame::new(2);
    let swarm = clockwork(&mut t, "Clockwork Swarm");
    let n = t.counters(swarm, PLUS1_0);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(swarm, PLUS1_0), n);
}

#[test]
fn it_can_attack_or_block_with_no_counters() {
    cr!("508.1a", "509.1a");
    ruling!(
        "Clockwork Avian",
        "Can attack or block even if it has no counters."
    );
    ruling!(
        "Clockwork Swarm",
        "Can attack or block even if it has no counters."
    );
    let mut t = TestGame::new(2);
    let avian = clockwork(&mut t, "Clockwork Avian");
    remove(&mut t, avian, 4);
    assert_eq!(t.pt(avian), (0, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, avian));
    // Blocking: P1's Clockwork Swarm with no counters blocks P0's Grizzly Bears.
    let mut t = TestGame::new(2);
    let swarm = t.enter(P1, "Clockwork Swarm");
    t.resolve_all();
    let n = t.counters(swarm, PLUS1_0);
    remove(&mut t, swarm, n);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(crate::r_s21_common::legal_blocks(&mut t, P1, &[(swarm, bears)]));
    block_and_finish(&mut t, P1, &[(swarm, bears)]);
    assert_eq!(t.life(P1), 20);
}
