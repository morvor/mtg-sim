//! Rulings batch P109 — removal with several targets: the same object can't be chosen
//! twice for one word "target" (CR 115.3), a spell that needs N targets can't be cast
//! without N legal ones (CR 601.2c), the remaining legal targets are still affected
//! (CR 608.2b), costs paid while casting (CR 601.2b, 601.2h), and an attack trigger that
//! resolves before blockers are declared (CR 508.2, 509.1).

use crate::r_p009_common::{dmg, kill, obj, pl};
use crate::r_s01_common::{attack_with, supported};
use crate::r_s05_common::move_to;
use crate::r_s17_common::enter_transformed;
use crate::r_s25_common::{lands_for_cost, targets_of};
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts the real card `name` for P0 with lands for its cost and the given targets.
fn try_cast(
    t: &mut TestGame,
    name: &str,
    targets: &[Entity],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    lands_for_cost(t, P0, name);
    let card = t.hand(P0, name);
    let r = t.cast(P0, card).targets(targets).try_go();
    t.clear_answers();
    r
}

/// The distinct targets of a stack object.
fn distinct_targets(t: &TestGame, s: ObjectId) -> usize {
    let mut v = targets_of(t, s);
    let n = v.len();
    v.sort_by_key(|e| format!("{e:?}"));
    v.dedup();
    assert_eq!(v.len(), n, "a target was chosen twice: {v:?}");
    n
}

/// One "two targets" spell: its name, a maker of a legal target, and the outcome check for
/// a target that was affected.
struct TwoTargets {
    name: &'static str,
    /// The real card each target is (on P1's battlefield).
    victim: &'static str,
    /// Whether the spell needs exactly two targets (false: "up to two").
    exactly_two: bool,
    /// Whether players are legal targets too (then there's always a second legal target).
    any_target: bool,
}

/// The same object can't be both targets; with two different ones both are affected; with
/// only one legal target, a "two targets" spell can't be cast.
fn two_different_targets(c: &TwoTargets) {
    supported(c.name);
    // Two different targets: both affected.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, c.victim);
    let b = t.battlefield(P1, c.victim);
    let s = try_cast(&mut t, c.name, &[obj(a), obj(b)]).expect(c.name);
    assert_eq!(distinct_targets(&t, s), 2, "{}", c.name);
    t.resolve_all();
    assert!(
        !t.on_battlefield(a) && !t.on_battlefield(b),
        "{}: both affected",
        c.name
    );
    // The same object twice isn't accepted.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, c.victim);
    let b = t.battlefield(P1, c.victim);
    if let Ok(s) = try_cast(&mut t, c.name, &[obj(a), obj(a)]) {
        distinct_targets(&t, s);
        let _ = b;
    }
    // Only one legal target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, c.victim);
    let r = try_cast(&mut t, c.name, &[obj(a), obj(a)]);
    if c.any_target {
        let s = r.expect(c.name);
        assert_eq!(distinct_targets(&t, s), 2, "{}: two legal targets", c.name);
    } else if c.exactly_two {
        assert!(r.is_err(), "{}: can't be cast with one legal target", c.name);
        assert!(t.in_hand(P0, c.name), "{}", c.name);
    } else {
        // "Up to two": the doubled answer was rejected; a single target is fine.
        drop(r);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, c.victim);
        let s = try_cast(&mut t, c.name, &[obj(a)]).expect(c.name);
        assert_eq!(distinct_targets(&t, s), 1, "{}", c.name);
    }
}

#[test]
fn two_target_spells_need_two_different_targets() {
    cr!("115.3", "601.2c");
    ruling!("Rack and Ruin", "Must target two different artifacts.");
    two_different_targets(&TwoTargets {
        name: "Rack and Ruin",
        victim: "Ornithopter",
        any_target: false,
        exactly_two: true,
    });
    ruling!(
        "Swelter",
        "You can’t cast this spell unless you have two different legal targets."
    );
    two_different_targets(&TwoTargets {
        name: "Swelter",
        victim: "Grizzly Bears",
        any_target: false,
        exactly_two: true,
    });
    ruling!(
        "Pinnacle of Rage",
        "You must choose two legal targets to cast Pinnacle of Rage."
    );
    two_different_targets(&TwoTargets {
        name: "Pinnacle of Rage",
        victim: "Grizzly Bears",
        any_target: true,
        exactly_two: true,
    });
    ruling!(
        "Furious Reprisal",
        "You can't target the same permanent or player twice to have one recipient be dealt 4 damage."
    );
    two_different_targets(&TwoTargets {
        name: "Furious Reprisal",
        victim: "Grizzly Bears",
        any_target: true,
        exactly_two: true,
    });
    ruling!(
        "Curtains' Call",
        "You can't cast Curtains' Call without two creatures to target."
    );
    two_different_targets(&TwoTargets {
        name: "Curtains' Call",
        victim: "Grizzly Bears",
        any_target: false,
        exactly_two: true,
    });
    ruling!(
        "Dual Shot",
        "You can’t target the same creature twice to have Dual Shot deal 2 damage to it."
    );
    two_different_targets(&TwoTargets {
        name: "Dual Shot",
        victim: "Llanowar Elves",
        any_target: false,
        exactly_two: false,
    });
}

#[test]
fn furious_reprisal_never_deals_four_to_one_recipient() {
    cr!("115.3");
    ruling!(
        "Furious Reprisal",
        "You can't target the same permanent or player twice to have one recipient be dealt 4 damage."
    );
    // A player and a creature: 2 each; the player can't be chosen twice.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let s = try_cast(&mut t, "Furious Reprisal", &[pl(P1), obj(wurm)]).unwrap();
    assert_eq!(distinct_targets(&t, s), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(dmg(&t, wurm), 2);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Craw Wurm");
    if let Ok(s) = try_cast(&mut t, "Furious Reprisal", &[pl(P1), pl(P1)]) {
        distinct_targets(&t, s);
        t.resolve_all();
        assert!(t.life(P1) >= 18, "never 4 to one player");
    }
}

#[test]
fn the_remaining_targets_are_still_destroyed() {
    cr!("608.2b");
    ruling!(
        "Curtains' Call",
        "If one becomes an illegal target after you've cast Curtains' Call, the other will still be destroyed."
    );
    supported("Curtains' Call");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    try_cast(&mut t, "Curtains' Call", &[obj(a), obj(b)]).unwrap();
    // Bounce one of them in response (a new object: an illegal target).
    move_to(&mut t, a, Zone::Hand(P1));
    t.resolve_all();
    assert!(!t.on_battlefield(b), "the other is still destroyed");
    assert!(t.in_hand(P1, "Grizzly Bears"));

    ruling!(
        "Hex",
        "You must target six different creatures. If you can't, you can't cast Hex. If some of the creatures become illegal targets before the spell resolves, Hex will still destroy the rest of them."
    );
    supported("Hex");
    // Five creatures: Hex can't be cast.
    let mut t = TestGame::new(2);
    let five: Vec<Entity> = (0..5)
        .map(|_| obj(t.battlefield(P1, "Grizzly Bears")))
        .collect();
    let mut twice = five.clone();
    twice.push(five[0]);
    assert!(try_cast(&mut t, "Hex", &twice).is_err());
    assert!(t.in_hand(P0, "Hex"));
    // Six: two leave in response, the other four are destroyed.
    let mut t = TestGame::new(2);
    let six: Vec<ObjectId> = (0..6).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    let es: Vec<Entity> = six.iter().map(|b| obj(*b)).collect();
    let s = try_cast(&mut t, "Hex", &es).unwrap();
    assert_eq!(distinct_targets(&t, s), 6);
    for b in &six[..2] {
        move_to(&mut t, *b, Zone::Hand(P1));
    }
    t.resolve_all();
    assert!(six[2..].iter().all(|b| !t.on_battlefield(*b)));
    assert_eq!(t.g.find_in_zone(Zone::Hand(P1), "Grizzly Bears").len(), 2);
    let _ = kill;
}

#[test]
fn eliminate_the_competition_sacrifices_as_it_is_cast() {
    cr!("601.2b", "601.2h", "118.9", "107.3a");
    supported("Eliminate the Competition");
    ruling!(
        "Eliminate the Competition",
        "Once you announce that you're casting Eliminate the Competition, players can't try to remove creatures you'd like to sacrifice."
    );
    // X = 2: the sacrifices are part of casting, so they're already gone when anyone
    // first gets priority with the spell on the stack.
    let mut t = TestGame::new(2);
    let mine: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let theirs: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Hill Giant")).collect();
    lands_for_cost(&mut t, P0, "Eliminate the Competition");
    let card = t.hand(P0, "Eliminate the Competition");
    t.answer_choose(P0, &[obj(mine[0]), obj(mine[1])]);
    t.cast(P0, card)
        .x(2)
        .targets(&[obj(theirs[0]), obj(theirs[1])])
        .go();
    t.clear_answers();
    assert_eq!(t.stack_len(), 1);
    assert!(mine.iter().all(|b| !t.on_battlefield(*b)), "sacrificed while casting");
    t.resolve_all();
    assert!(theirs.iter().all(|g| !t.on_battlefield(*g)));

    ruling!(
        "Eliminate the Competition",
        "If you somehow cast Eliminate the Competition without paying its mana cost, you'll still choose a value for X and sacrifice X creatures."
    );
    let mut t = TestGame::new(2);
    let mine: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let theirs: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Hill Giant")).collect();
    let card = t.hand(P0, "Eliminate the Competition");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![card],
        mtg_engine::ability::Duration::EndOfTurn,
        true,
        None,
    );
    t.answer_choose(P0, &[obj(mine[0]), obj(mine[1])]);
    let s = t
        .cast(P0, card)
        .method(CastMethod::Free)
        .x(2)
        .targets(&[obj(theirs[0]), obj(theirs[1])])
        .go();
    t.clear_answers();
    assert_eq!(t.g.obj(s).stack.as_deref().and_then(|si| si.x), Some(2));
    assert!(mine.iter().all(|b| !t.on_battlefield(*b)), "X creatures sacrificed");
    t.resolve_all();
    assert!(theirs.iter().all(|g| !t.on_battlefield(*g)));
    let _ = Answer::Default;
}

#[test]
fn erupting_dreadwolf_kills_a_blocker_before_blocks() {
    cr!("508.2", "509.1", "603.3");
    ruling!(
        "Smoldering Werewolf // Erupting Dreadwolf",
        "Erupting Dreadwolf's triggered ability resolves before blockers are chosen. A creature dealt lethal damage this way won't be around to block."
    );
    supported("Smoldering Werewolf // Erupting Dreadwolf");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wolf = enter_transformed(&mut t, P0, "Smoldering Werewolf // Erupting Dreadwolf");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let w = t.g.current(wolf);
    t.g.obj_mut(w).summoning_sick = false;
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(w, pl(P1))]);
    // The trigger is on the stack in the declare attackers step; it resolves there.
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.on_battlefield(bears), "dead before blockers are declared");
}
