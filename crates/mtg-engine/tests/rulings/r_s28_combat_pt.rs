//! Rulings batch S28 — Sentinel ("{0}: This creature's base toughness becomes equal to 1
//! plus the power of target creature blocking or blocked by this creature.") and Sworn
//! Defender ("{1}: This creature's power becomes the toughness of target creature blocking
//! or being blocked by this creature minus 1 until end of turn, and its toughness becomes
//! 1 plus the power of that creature until end of turn."): effects that set power and
//! toughness in layer 7b (CR 613.4b), so modifying effects, counters (7c) and switches
//! (7d) still apply on top of them.

use crate::r_s01_common::{attack_with, supported};
use crate::r_s21_common::go_to;
use crate::r_s28_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's `name` attacks P1 and is blocked by P1's `blocker`; the game is in the declare
/// blockers step.
fn blocked_by(t: &mut TestGame, name: &str, blocker: &str) -> (ObjectId, ObjectId) {
    supported(name);
    let me = t.battlefield(P0, name);
    let them = t.battlefield(P1, blocker);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(t, &[(me, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(them, me)]),
    );
    go_to(t, Step::DeclareBlockers);
    (me, them)
}

fn cast_at(t: &mut TestGame, name: &str, target: ObjectId) {
    t.answer_targets(P0, &[Entity::Object(target)]);
    cast_card(t, P0, name);
    t.resolve_all();
}

#[test]
fn sentinels_toughness_is_set_in_layer_7b_under_pumps_counters_and_switches() {
    cr!("613.4b", "613.4c", "613.4d", "608.2h");
    ruling!(
        "Sentinel",
        "This card’s effect is always applied in (b), which means that effects applied in sublayer (c), (d), or (e) will not be overwritten; they will be applied to the new value."
    );
    let mut t = TestGame::new(2);
    let (sentinel, bears) = blocked_by(&mut t, "Sentinel", "Grizzly Bears");
    // Giant Growth and a +1/+1 counter before the ability.
    cast_at(&mut t, "Giant Growth", sentinel);
    t.g.add_counters(Entity::Object(sentinel), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(sentinel), (5, 5));
    // Base toughness 1 + 2 (the Bears' power): 1/3, then +3/+3 and +1/+1.
    t.activate(P0, sentinel, 0, &[Entity::Object(bears)])
        .expect("activate Sentinel");
    t.resolve_all();
    assert_eq!(t.pt(sentinel), (5, 7));
    // The Bears' power was determined as the ability resolved: a later counter on them
    // doesn't change the Sentinel's toughness.
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(sentinel), (5, 7));
    // A switch applies on top of it.
    cast_at(&mut t, "Twisted Image", sentinel);
    assert_eq!(t.pt(sentinel), (7, 5));
    // The new base toughness lasts indefinitely.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.pt(sentinel), (2, 4));
}

#[test]
fn sworn_defenders_pt_is_set_in_layer_7b_under_pumps_and_counters() {
    cr!("613.4b", "613.4c", "608.2h");
    ruling!(
        "Sworn Defender",
        "This card’s effect is always applied in (b), which means that effects applied in sublayer (c), (d), or (e) will not be overwritten; they will be applied to the new value."
    );
    let mut t = TestGame::new(2);
    let (defender, giant) = blocked_by(&mut t, "Sworn Defender", "Hill Giant");
    cast_at(&mut t, "Giant Growth", defender);
    assert_eq!(t.pt(defender), (4, 6));
    // Power 3 - 1 = 2, toughness 1 + 3 = 4, then +3/+3.
    t.lands(P0, "Wastes", 1);
    t.activate(P0, defender, 0, &[Entity::Object(giant)])
        .expect("activate Sworn Defender");
    t.resolve_all();
    assert_eq!(t.pt(defender), (5, 7));
    // The Giant's power and toughness were determined as the ability resolved.
    t.g.add_counters(Entity::Object(giant), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(giant), (4, 4));
    assert_eq!(t.pt(defender), (5, 7));
    t.g.add_counters(Entity::Object(defender), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(defender), (6, 8));
    // Combat damage: 6 to the Giant, 4 to the Defender (toughness 8).
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(defender));
    // Until end of turn: back to 1/3 (+1/+1).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(defender), (2, 4));
}
