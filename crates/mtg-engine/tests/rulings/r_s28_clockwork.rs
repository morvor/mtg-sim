//! Rulings batch S28 — the Clockwork creatures: "This creature enters with N +1/+0
//! counters on it. At end of combat, if this creature attacked or blocked this combat,
//! remove a +1/+0 counter from it. {X}, {T}: Put up to X +1/+0 counters on this creature.
//! This ability can't cause the total number of +1/+0 counters on this creature to be
//! greater than N. Activate only during your upkeep."

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::can_attack;
use crate::r_s28_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PLUS1_0: &str = "+1/+0";

/// A Clockwork creature for P0 that entered with its counters and isn't summoning sick.
fn clockwork(t: &mut TestGame, name: &str) -> ObjectId {
    supported(name);
    let id = t.enter(P0, name);
    t.g.objects[id.0 as usize].summoning_sick = false;
    id
}

#[test]
fn a_clockwork_creature_with_no_counters_can_attack_and_block() {
    cr!("508.1a", "509.1a", "122.1");
    ruling!(
        "Clockwork Steed",
        "Can attack or block even if it has no counters."
    );
    let mut t = TestGame::new(2);
    let steed = clockwork(&mut t, "Clockwork Steed");
    assert_eq!(t.counters(steed, PLUS1_0), 4);
    assert_eq!(t.pt(steed), (4, 3));
    t.g.remove_counters(Entity::Object(steed), PLUS1_0, 4);
    t.g.recompute();
    assert_eq!(t.pt(steed), (0, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, steed));
    attack_with(&mut t, &[(steed, Entity::Player(P1))]);
    assert!(crate::r_s10_common::attacking(&t, steed));
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.counters(steed, PLUS1_0), 0);
    // It can block too.
    let mut t = TestGame::new(2);
    let steed = clockwork(&mut t, "Clockwork Steed");
    t.g.remove_counters(Entity::Object(steed), PLUS1_0, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    assert!(crate::r_s21_common::legal_blocks(&mut t, P0, &[(steed, bears)]));
}

#[test]
fn a_clockwork_creature_loses_a_counter_even_if_its_damage_is_prevented() {
    cr!("511.2", "603.4", "615.1");
    ruling!(
        "Clockwork Steed",
        "Loses a counter even if it is affected by a Fog-like effect which prevents it from dealing damage."
    );
    supported("Fog");
    let mut t = TestGame::new(2);
    let steed = clockwork(&mut t, "Clockwork Steed");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(steed, Entity::Player(P1))]);
    // Fog: "Prevent all combat damage that would be dealt this turn."
    cast_card(&mut t, P1, "Fog");
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(steed, PLUS1_0), 3);
    // A Clockwork creature that didn't attack or block keeps its counters.
    let mut t = TestGame::new(2);
    let steed = clockwork(&mut t, "Clockwork Steed");
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::End);
    assert_eq!(t.counters(steed, PLUS1_0), 4);
}

/// Activates the Clockwork creature's last ability during P0's upkeep with X = `x`,
/// choosing to put `put` counters as it resolves.
fn wind(t: &mut TestGame, id: ObjectId, x: i64, put: i64) {
    t.set_step(P0, Step::Upkeep);
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer(P0, DecisionKind::Number, Answer::Number(put));
    t.activate(P0, id, 0, &[]).expect("activate during the upkeep");
    t.resolve_all();
    let now = t.g.current(id);
    t.g.objects[now.0 as usize].tapped = false;
}

#[test]
fn you_can_put_fewer_than_x_counters() {
    cr!("107.3a", "122.1");
    ruling!(
        "Clockwork Beast",
        "Clockwork Beast’s last ability resolves, you can choose to put fewer than X +1/+0 counters on it."
    );
    let mut t = TestGame::new(2);
    let beast = clockwork(&mut t, "Clockwork Beast");
    t.g.remove_counters(Entity::Object(beast), PLUS1_0, 5);
    wind(&mut t, beast, 3, 1);
    assert_eq!(t.counters(beast, PLUS1_0), 3);
    // Only during the upkeep.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    assert!(t.activate(P0, beast, 0, &[]).is_err());
}

#[test]
fn the_ability_cant_raise_the_total_above_the_maximum() {
    cr!("107.3a", "122.1");
    ruling!(
        "Clockwork Beast",
        "If Clockwork Beast has seven or fewer +1/+0 counters on it when its last ability resolves, it can wind up a maximum of seven such counters on it. If it has seven or more +1/+0 counters on it, the ability will have no effect."
    );
    let mut t = TestGame::new(2);
    let beast = clockwork(&mut t, "Clockwork Beast");
    t.g.remove_counters(Entity::Object(beast), PLUS1_0, 2);
    // X = 4 with five counters: at most two more.
    let from = t.asked().len();
    wind(&mut t, beast, 4, 2);
    let offered: Vec<(i64, i64)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseNumber { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![(0, 2)]);
    assert_eq!(t.counters(beast, PLUS1_0), 7);
    // With seven, nothing happens.
    wind(&mut t, beast, 2, 2);
    assert_eq!(t.counters(beast, PLUS1_0), 7);
}

#[test]
fn the_steed_and_the_avian_are_capped_at_four() {
    cr!("107.3a", "122.1");
    ruling!(
        "Clockwork Steed",
        "If Clockwork Steed has four or fewer +1/+0 counters on it when its last ability resolves, it can wind up with a maximum of four such counters on it. If it has four or more +1/+0 counters on it, the ability will have no effect."
    );
    ruling!(
        "Clockwork Avian",
        "If the ability to add counters resolves when there are already the maximum number of counters on it, any counters over the maximum are simply not added."
    );
    let mut t = TestGame::new(2);
    let steed = clockwork(&mut t, "Clockwork Steed");
    t.g.remove_counters(Entity::Object(steed), PLUS1_0, 1);
    wind(&mut t, steed, 3, 1);
    assert_eq!(t.counters(steed, PLUS1_0), 4);
    wind(&mut t, steed, 3, 3);
    assert_eq!(t.counters(steed, PLUS1_0), 4);
    let avian = clockwork(&mut t, "Clockwork Avian");
    assert_eq!(t.counters(avian, PLUS1_0), 4);
    wind(&mut t, avian, 2, 2);
    assert_eq!(t.counters(avian, PLUS1_0), 4);
}

#[test]
fn other_effects_can_put_more_counters_than_the_maximum() {
    cr!("122.1", "701.34a");
    ruling!(
        "Clockwork Beast",
        "Now, if some other spell or ability causes +1/+0 counters to be put on Clockwork Beast, it can wind up with more than seven such counters on it."
    );
    let mut t = TestGame::new(2);
    let beast = clockwork(&mut t, "Clockwork Beast");
    assert_eq!(t.counters(beast, PLUS1_0), 7);
    t.answer_choose(P0, &[Entity::Object(beast)]);
    cast_card(&mut t, P0, "Steady Progress");
    t.resolve_all();
    assert_eq!(t.counters(beast, PLUS1_0), 8);
    assert_eq!(t.pt(beast), (8, 4));
}
