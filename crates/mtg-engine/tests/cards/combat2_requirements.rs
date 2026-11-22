//! Attack requirements toward the active player's creatures and their follow-ups,
//! creatures controlled since the beginning of the turn (CR 302.6, 508.1d), and combat
//! referents (the attacking player, CR 506.2; creatures blocking or blocked by a creature,
//! CR 509.1g): Siren's Call, Maddening Imp, Norritt, Nettling Imp, Keldon Twilight, Total
//! War, Contested Game Ball, Souls of the Faultless, Trial // Error.

use crate::combat2_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The candidates offered for the last target choice of `p`.
fn offered_targets(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn sirens_call_makes_the_active_players_creatures_attack() {
    cr!("508.1d", "506.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.lands(P0, "Island", 1);
    let call = t.hand(P0, "Siren's Call");
    t.cast(P0, call).go();
    t.resolve();
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    // P0's creature isn't affected (it isn't P0's turn): P0's own turn, nothing required.
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
    let _ = mine;
}

#[test]
fn maddening_imp_destroys_the_creatures_that_didnt_attack() {
    cr!("508.1d", "603.7");
    ruling!(
        "Maddening Imp",
        "If a creature can’t attack because of some restriction, or because it is tapped, it is destroyed at the end of the turn."
    );
    supported("Maddening Imp");
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Maddening Imp");
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let tapped = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.g.tap(tapped);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P0, imp, 0, &[]).unwrap();
    t.resolve();
    // A creature entering later isn't one of "those creatures".
    let late = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    declare(
        &mut t,
        P1,
        &[(attacker, Entity::Player(P0)), (late, Entity::Player(P0))],
    );
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(attacker));
    assert!(!t.on_battlefield(tapped));
    // Walls aren't required to attack nor destroyed.
    assert!(t.on_battlefield(wall));
    assert!(t.on_battlefield(late));
}

#[test]
fn maddening_imp_only_during_an_opponents_turn_before_combat() {
    cr!("602.5b");
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Maddening Imp");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, imp, 0, &[]).is_err());
    t.set_step(P1, Step::PostcombatMain);
    assert!(t.activate(P0, imp, 0, &[]).is_err());
}

#[test]
fn norritt_targets_only_creatures_controlled_since_the_turn_began() {
    cr!("302.6", "115.1", "508.1d");
    supported("Norritt");
    let mut t = TestGame::new(2);
    let norritt = t.battlefield(P0, "Norritt");
    let old = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let mine = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let new = t.enter(P1, "Llanowar Elves");
    t.activate(P0, norritt, 1, &[Entity::Object(old)]).unwrap();
    let offered = offered_targets(&t, P0);
    assert!(offered.contains(&Entity::Object(old)));
    for not in [new, wall, mine] {
        assert!(!offered.contains(&Entity::Object(not)));
    }
    t.resolve();
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    declare(&mut t, P1, &[(old, Entity::Player(P0))]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(old));
}

#[test]
fn nettling_imp_destroys_a_creature_that_couldnt_attack() {
    cr!("508.1d", "603.7");
    ruling!(
        "Nettling Imp",
        "You can use this effect on a creature you know won’t be able to attack. For example, you can use it on a tapped creature."
    );
    ruling!(
        "Nettling Imp",
        "If the Imp leaves the battlefield before the end of the turn, the creature still is destroyed."
    );
    supported("Nettling Imp");
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Nettling Imp");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P0, imp, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    t.g.destroy(imp, None);
    t.settle();
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn keldon_twilight_sacrifices_a_creature_controlled_since_the_turn_began() {
    cr!("302.6", "701.21a");
    supported("Keldon Twilight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keldon Twilight");
    let old = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let new = t.enter(P1, "Llanowar Elves");
    // P1 would rather sacrifice the Elves, which isn't allowed.
    t.answer_choose(P1, &[Entity::Object(new)]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(old));
    assert!(t.on_battlefield(new));
}

#[test]
fn keldon_twilight_does_nothing_if_creatures_attacked() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keldon Twilight");
    let old = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, P1, &[(other, Entity::Player(P0))]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(old));
}

#[test]
fn total_war_destroys_creatures_that_stayed_home() {
    cr!("302.6", "508.1", "603.2");
    ruling!(
        "Total War",
        "Destroys creatures as a triggered ability on the declaration of attackers."
    );
    supported("Total War");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Total War");
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P0, "Wall of Wood");
    let tapped = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(tapped);
    let new = t.enter(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // A vigilant attacker stays untapped: it attacked.
    let angel = t.battlefield(P0, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(attacker, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(!t.obj_now(angel).tapped);
    assert!(t.on_battlefield(attacker));
    assert!(!t.on_battlefield(home));
    for alive in [wall, tapped, new, theirs, angel] {
        assert!(t.on_battlefield(alive));
    }
}

#[test]
fn contested_game_ball_goes_to_the_attacking_player() {
    cr!("506.2", "510.2");
    ruling!(
        "Contested Game Ball",
        "Contested Game Ball's triggered ability triggers only once whenever you're dealt combat damage"
    );
    supported("Contested Game Ball");
    let mut t = TestGame::new(2);
    let ball = t.battlefield(P1, "Contested Game Ball");
    t.g.tap(ball);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    // Two creatures deal combat damage at once: one trigger.
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(controller(&t, ball), P0);
    assert!(!t.obj_now(ball).tapped);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn souls_of_the_faultless_drains_the_attacking_player() {
    cr!("506.2", "510.2");
    supported("Souls of the Faultless");
    let mut t = TestGame::new(2);
    let souls = t.battlefield(P1, "Souls of the Faultless");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(souls, giant)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    assert_eq!(t.life(P0), 17);
}

fn trial(t: &mut TestGame, target: ObjectId) {
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let spell = t.hand(P0, "Trial // Error");
    t.cast(P0, spell)
        .method(CastMethod::Half(0))
        .target(target)
        .go();
    t.resolve();
}

#[test]
fn trial_returns_creatures_blocking_the_target() {
    cr!("509.1g", "709.3");
    supported("Trial // Error");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    let w3 = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    declare_blocks(&mut t, P1, &[(w1, a), (w2, a), (w3, b)]);
    trial(&mut t, a);
    assert!(!t.on_battlefield(w1) && !t.on_battlefield(w2));
    assert!(t.on_battlefield(w3) && t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn trial_returns_creatures_blocked_by_the_target() {
    cr!("509.1g");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let w = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    declare_blocks(&mut t, P1, &[(w, a)]);
    trial(&mut t, w);
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b) && t.on_battlefield(w));
}
