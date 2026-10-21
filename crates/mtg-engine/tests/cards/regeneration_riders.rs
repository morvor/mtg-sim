//! Regeneration (CR 701.19a, 701.19c) with riders: "When it regenerates this way, put a
//! -1/-1 counter on it." (a triggered ability when the shield is used), "You gain control
//! of that creature if it regenerates this way." (part of the regeneration), "Pay {1} for
//! each +1/+1 counter on ~: Regenerate ~."; and "can't be regenerated" after an instruction
//! that replaces an earlier one ("Threshold — If ..., instead destroy that creature. It
//! can't be regenerated.") or that names "the creature".

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn matopi_golem_gets_a_counter_when_it_regenerates() {
    cr!("701.19a", "603.12");
    assert_supported("Matopi Golem");
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Matopi Golem");
    t.lands(P0, "Plains", 1);
    t.activate(P0, golem, 0, &[]).unwrap();
    t.resolve();
    // Activating it doesn't regenerate it: no counter yet.
    assert_eq!(t.counters(golem, "-1/-1"), 0);
    t.g.destroy(golem, None);
    t.resolve_all();
    assert!(t.on_battlefield(golem));
    assert!(t.g.obj(golem).tapped);
    assert_eq!(t.counters(golem, "-1/-1"), 1);
    assert_eq!(t.pt(golem), (2, 2));
}

#[test]
fn soldevi_sentry_lets_the_chosen_opponent_draw_when_it_regenerates() {
    cr!("701.19a", "603.12");
    assert_supported("Soldevi Sentry");
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Soldevi Sentry");
    t.lands(P0, "Plains", 1);
    t.activate(P0, sentry, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    t.g.destroy(sentry, None);
    t.resolve_all();
    assert!(t.on_battlefield(sentry));
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn debt_of_loyalty_gains_control_as_the_creature_regenerates() {
    cr!("701.19a");
    ruling!("Debt of Loyalty", "instead it regenerates and you gain control of it");
    assert_supported("Debt of Loyalty");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    let debt = t.hand(P0, "Debt of Loyalty");
    t.cast(P0, debt).target(giant).go();
    t.resolve();
    // Nothing visible until it would be destroyed.
    assert_eq!(t.g.obj(giant).controller, P1);
    t.g.destroy(giant, None);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.g.obj(giant).controller, P0);
    assert!(t.g.obj(giant).tapped);
}

#[test]
fn debt_of_loyalty_without_its_shield_used_changes_nothing() {
    cr!("701.19a", "616.1");
    ruling!("Debt of Loyalty", "If the player chooses to use the other shield");
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P1, "Matopi Golem");
    t.lands(P1, "Plains", 1);
    t.lands(P0, "Plains", 3);
    let debt = t.hand(P0, "Debt of Loyalty");
    t.cast(P0, debt).target(golem).go();
    t.resolve();
    // The Golem's own shield, created later.
    t.activate(P1, golem, 0, &[]).unwrap();
    t.resolve();
    // P1 chooses the Golem's shield (the second of the two replacement effects).
    t.answer(P1, DecisionKind::Replacement, decision::Answer::Index(1));
    t.g.destroy(golem, None);
    t.resolve_all();
    assert!(t.on_battlefield(golem));
    // The Golem's shield was used: no control change, and its rider triggered.
    assert_eq!(t.g.obj(golem).controller, P1);
    assert_eq!(t.counters(golem, "-1/-1"), 1);
    // The Debt of Loyalty shield is still there for the rest of the turn.
    t.g.destroy(golem, None);
    t.resolve_all();
    assert!(t.on_battlefield(golem));
    assert_eq!(t.g.obj(golem).controller, P0);
}

#[test]
fn skeleton_scavengers_pays_one_for_each_counter() {
    cr!("701.19a", "118.3");
    assert_supported("Skeleton Scavengers");
    let mut t = TestGame::new(2);
    let skel = t.enter(P0, "Skeleton Scavengers");
    assert_eq!(t.counters(skel, "+1/+1"), 1);
    // One counter: {1}.
    assert!(t.activate(P0, skel, 0, &[]).is_err());
    t.lands(P0, "Swamp", 1);
    t.activate(P0, skel, 0, &[]).unwrap();
    t.resolve();
    t.g.destroy(skel, None);
    t.resolve_all();
    assert!(t.on_battlefield(skel));
    assert_eq!(t.counters(skel, "+1/+1"), 2);
    // Two counters now: {2}, and only one Swamp (untapped next turn would help).
    t.lands(P0, "Swamp", 1);
    assert!(t.activate(P0, skel, 0, &[]).is_err());
}

#[test]
fn toxic_stench_with_threshold_destroys_and_it_cant_be_regenerated() {
    cr!("701.19c", "614.1a");
    assert_supported("Toxic Stench");
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P1, "Matopi Golem");
    t.lands(P1, "Plains", 1);
    t.activate(P1, golem, 0, &[]).unwrap();
    t.resolve();
    for _ in 0..7 {
        t.graveyard(P0, "Swamp");
    }
    t.lands(P0, "Swamp", 2);
    let ts = t.hand(P0, "Toxic Stench");
    t.cast(P0, ts).target(golem).go();
    t.resolve_all();
    assert!(!t.on_battlefield(golem));
    assert!(t.in_graveyard(P1, "Matopi Golem"));
}

#[test]
fn kirtars_wrath_with_threshold_creatures_destroyed_this_way_cant_be_regenerated() {
    cr!("701.19c", "614.1a");
    assert_supported("Kirtar's Wrath");
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P1, "Matopi Golem");
    t.lands(P1, "Plains", 1);
    t.activate(P1, golem, 0, &[]).unwrap();
    t.resolve();
    for _ in 0..7 {
        t.graveyard(P0, "Plains");
    }
    t.lands(P0, "Plains", 6);
    let kw = t.hand(P0, "Kirtar's Wrath");
    t.cast(P0, kw).go();
    t.resolve_all();
    assert!(!t.on_battlefield(golem));
    assert_eq!(
        t.g.permanents().filter(|o| o.chars.has_subtype("Spirit")).count(),
        2
    );
}

#[test]
fn mephitic_ooze_destroys_the_creature_it_damaged_without_regeneration() {
    cr!("701.19c", "510.2");
    assert_supported("Mephitic Ooze");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Mephitic Ooze");
    t.battlefield(P0, "Ornithopter");
    let golem = t.battlefield(P1, "Matopi Golem");
    t.lands(P1, "Plains", 1);
    t.set_step(P0, turn::Step::PrecombatMain);
    t.activate(P1, golem, 0, &[]).unwrap();
    t.resolve();
    t.attack(&[(ooze, Entity::Player(P1))], &[(golem, ooze)]);
    assert!(!t.on_battlefield(golem));
}
