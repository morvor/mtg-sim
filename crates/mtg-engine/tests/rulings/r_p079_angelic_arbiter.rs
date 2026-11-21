//! Rulings batch P079 — Angelic Arbiter: "Each opponent who cast a spell this turn can't
//! attack with creatures. Each opponent who attacked with a creature this turn can't cast
//! spells." (pattern `r508_attacked_or_cast_this_turn_restrictions`; CR 508.1c, 601.3).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_attack, can_play_land, destroy};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 controls Angelic Arbiter (unless `arbiter` is false); it's P1's precombat main
/// phase, P1 has a Grizzly Bears and a Shock with a Mountain for it.
fn p1_turn(arbiter: bool) -> (TestGame, Option<ObjectId>, ObjectId) {
    let mut t = TestGame::new(2);
    let a = arbiter.then(|| t.battlefield(P0, "Angelic Arbiter"));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    (t, a, bears)
}

fn shock(t: &mut TestGame) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    let r = cast_card(t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    r
}

#[test]
fn casting_a_spell_stops_attacking_and_attacking_stops_casting() {
    cr!("508.1c", "601.3", "611.3a");
    ruling!(
        "Angelic Arbiter",
        "Angelic Arbiter's last ability applies if an opponent attacked with at least one creature during the current turn."
    );
    ruling!(
        "Angelic Arbiter",
        "During an opponent's turn, that opponent may either cast spells or attack with creatures, but not both"
    );
    supported("Angelic Arbiter");
    // P1 casts Shock: the Bears can't attack. P1 may still play a land.
    let (mut t, _, bears) = p1_turn(true);
    shock(&mut t).unwrap();
    assert_eq!(t.life(P0), 18);
    let forest = t.hand(P1, "Forest");
    assert!(can_play_land(&mut t, P1, forest), "playing lands is fine");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
    // P1 attacks first: no spells afterwards (Shock can't be cast).
    let (mut t, _, bears) = p1_turn(true);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    let card = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(
        !can_cast_now(&mut t, P1, card),
        "attacked with a creature this turn"
    );
    // Without the Arbiter, both.
    let (mut t, _, bears) = p1_turn(false);
    shock(&mut t).unwrap();
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
}

#[test]
fn angelic_arbiter_does_nothing_during_your_turn() {
    cr!("508.1c", "601.3");
    ruling!(
        "Angelic Arbiter",
        "During your turn, Angelic Arbiter's abilities have no effect on the game."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Arbiter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    cast_card(&mut t, P0, "Shock", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let card = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    assert!(can_cast_now(&mut t, P0, card));
}

#[test]
fn angelic_arbiter_counts_actions_before_it_entered_and_stops_when_it_leaves() {
    cr!("611.3a", "508.1c");
    ruling!(
        "Angelic Arbiter",
        "If Angelic Arbiter enters during an opponent's turn, its abilities take actions that player performed earlier in the turn into account"
    );
    ruling!(
        "Angelic Arbiter",
        "If Angelic Arbiter leaves the battlefield during an opponent's turn, its abilities cease to affect the game."
    );
    // P1 casts Shock, then the Arbiter enters: the Bears can't attack.
    let (mut t, _, bears) = p1_turn(false);
    shock(&mut t).unwrap();
    let arbiter = t.battlefield(P0, "Angelic Arbiter");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
    // The Arbiter is destroyed: they can.
    destroy(&mut t, arbiter);
    assert!(can_attack(&mut t, bears));
}
