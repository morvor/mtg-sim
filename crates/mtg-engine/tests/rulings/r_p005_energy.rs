//! Rulings batch P005 — paying energy and sacrificing as abilities resolve: "you may pay
//! {E}" chosen once on resolution (CR 107.14, 608.2c) with "if you do" and reflexive "when
//! you do" (CR 603.12) follow-ups, abilities whose targets all became illegal (CR 608.2b),
//! divided damage fixed as the ability is put on the stack (CR 601.2d, 603.3d), sacrifices
//! chosen on resolution, and "each opponent" in Two-Headed Giant (CR 810.9).

use crate::r_p006_common::{
    in_graveyard, mark, obj, pl, priority_since, targets_of, top, yes_no_since,
};
use crate::r_p044_common::{kill, two_headed_giant};
use crate::r_p046_common::bounce;
use crate::r_p109_common::{divide, division_of};
use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s04_common::cycle;
use crate::r_s28_common::energy;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn give_energy(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.add_counters(Entity::Player(p), "energy", n, None);
}

/// The total number of counters (of any kind) on a permanent.
fn all_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counters.values().sum()
}

#[test]
fn behemoth_with_an_illegal_target_doesnt_resolve_and_nothing_is_paid() {
    cr!("608.2b");
    ruling!(
        "Behemoth of Vault 0",
        "If a spell or ability with one or more targets states that you “may pay” some amount of {E}, and each permanent that it targets has become an illegal target, the spell or ability won’t resolve. You can’t pay any {E} even if you want to."
    );
    supported("Behemoth of Vault 0");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let behemoth = t.battlefield(P0, "Behemoth of Vault 0");
    give_energy(&mut t, P0, 6);
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, behemoth);
    assert_eq!(targets_of(&t, top(&t)), vec![obj(giant)]);
    // The Hill Giant is returned to its owner's hand in response.
    bounce(&mut t, giant);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_since(&t, P0, from), 0, "never asked to pay");
    assert_eq!(energy(&t, P0), 6);
    assert!(t.g.stack.is_empty());
}

#[test]
fn electrosiphon_with_an_illegal_target_gives_no_energy() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Electrosiphon",
        "Some spells and abilities that give you {E} may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won’t resolve. You won’t get any {E}."
    );
    supported("Electrosiphon");
    for countered_first in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        give_mana_for(&mut t, P1, "Lava Axe");
        let axe = t.hand(P1, "Lava Axe");
        let axe = t.cast(P1, axe).target(pl(P0)).go();
        give_mana_for(&mut t, P0, "Electrosiphon");
        let siphon = t.hand(P0, "Electrosiphon");
        t.cast(P0, siphon).target(axe).go();
        if countered_first {
            // Counterspell, cast above Electrosiphon, counters Lava Axe first.
            give_mana_for(&mut t, P0, "Counterspell");
            let cs = t.hand(P0, "Counterspell");
            t.cast(P0, cs).target(axe).go();
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Lava Axe"));
        assert_eq!(t.life(P0), 20);
        // Lava Axe's mana value is 5.
        let expected = if countered_first { 0 } else { 5 };
        assert_eq!(
            energy(&t, P0),
            expected,
            "countered first: {countered_first}"
        );
    }
}

#[test]
fn may_pay_energy_is_one_payment_at_most() {
    cr!("107.14", "608.2c");
    ruling!(
        "Assaultron Dominator",
        "Some triggered abilities state that you “may pay” a certain amount of {E}. You can’t pay that amount multiple times to multiply the effect. You simply choose whether or not to pay that amount of {E} as the ability resolves."
    );
    supported("Assaultron Dominator");
    supported("Sentry Bot");
    // Assaultron Dominator: "Whenever an artifact creature you control attacks, you may pay
    // {E}. If you do, put your choice of a +1/+1, first strike, or trample counter on that
    // creature."
    let mut t = TestGame::new(2);
    let dominator = t.battlefield(P0, "Assaultron Dominator");
    give_energy(&mut t, P0, 5);
    attack_with(&mut t, &[(dominator, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(yes_no_since(&t, P0, from), 1);
    assert_eq!(energy(&t, P0), 4);
    assert_eq!(all_counters(&t, dominator), 1);
    // Sentry Bot: "At the beginning of combat on your turn, you may pay {E}{E}{E}. If you
    // do, put a +1/+1 counter on each creature you control."
    let mut t = TestGame::new(2);
    let bot = t.battlefield(P0, "Sentry Bot");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_energy(&mut t, P0, 9);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(yes_no_since(&t, P0, from), 1);
    assert_eq!(energy(&t, P0), 6);
    assert_eq!(t.counters(bot, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

/// Whether a decision is a priority decision.
fn is_priority(d: &Decision) -> bool {
    matches!(d, Decision::Priority { .. })
}

#[test]
fn if_you_do_and_when_you_do_leave_no_window_after_paying() {
    cr!("608.2c", "603.12", "603.3d");
    ruling!(
        "Behemoth of Vault 0",
        "Some triggered abilities that state that you “may pay” a certain amount of {E} describe an effect that happens “If you do.” In that case, no player may take actions to try to stop the ability’s effect after you make your choice. If the payment is followed by the phrase “When you do,” then you’ll choose any targets for that reflexive triggered ability and put it on the stack before players can take actions."
    );
    // "If you do" (Assaultron Dominator): the counter is put on as part of the same
    // resolution; no one gets priority between the choice and the effect.
    let mut t = TestGame::new(2);
    let dominator = t.battlefield(P0, "Assaultron Dominator");
    give_energy(&mut t, P0, 1);
    attack_with(&mut t, &[(dominator, pl(P1))]);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.g.resolve_top();
    assert_eq!(all_counters(&t, dominator), 1);
    assert_eq!(yes_no_since(&t, P0, from), 1);
    assert!(!t.asked()[from..].iter().any(|(_, d)| is_priority(d)));
    // "When you do" (Behemoth of Vault 0): the reflexive trigger is put on the stack
    // before anyone gets priority; the Hill Giant is destroyed only when it resolves.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let behemoth = t.battlefield(P0, "Behemoth of Vault 0");
    give_energy(&mut t, P0, 6);
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, behemoth);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(t.stack_len(), 1, "the reflexive trigger");
    assert!(t.on_battlefield(giant));
    assert!(priority_since(&t, from).is_empty());
    t.resolve();
    assert!(in_graveyard(&t, giant));
    // A reflexive trigger with a target (Cyclops Superconductor: "When this creature dies,
    // you may pay {E}{E}{E}. When you do, this creature deals damage equal to its power
    // to any target."): its target is chosen as it's put on the stack, still before
    // anyone gets priority.
    supported("Cyclops Superconductor");
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Cyclops Superconductor");
    give_energy(&mut t, P0, 3);
    kill(&mut t, cyclops);
    assert_eq!(t.stack_len(), 1);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_of(&t, top(&t)), vec![pl(P1)]);
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    assert!(!asked.iter().any(|(_, d)| is_priority(d)));
    t.resolve();
    // Its power as it last existed on the battlefield: 2.
    assert_eq!(t.life(P1), 18);
}

/// Kuldotha Flamefiend enters under P0's control (with an Ornithopter to sacrifice) and
/// its trigger is put on the stack with the given targets and division.
fn flamefiend_enters(t: &mut TestGame, targets: &[Entity], division: &[i64]) -> ObjectId {
    supported("Kuldotha Flamefiend");
    let orni = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, targets);
    divide(t, P0, division);
    t.enter(P0, "Kuldotha Flamefiend");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    orni
}

#[test]
fn kuldotha_flamefiend_divides_as_the_trigger_goes_on_the_stack() {
    cr!("603.3d", "601.2d");
    ruling!(
        "Kuldotha Flamefiend",
        "You divide the damage as Kuldotha Flamefiend's triggered ability is put onto the stack, not as it resolves. Each target must be assigned at least 1 damage. You can't choose more than 4 targets and assign 0 damage to a target."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = mark(&t);
    let orni = flamefiend_enters(&mut t, &[pl(P1), obj(bears), obj(giant)], &[1, 2, 1]);
    // Up to four targets, each dealt at least 1.
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { max: 4, .. })));
    assert!(asked.iter().any(|(_, d)| matches!(
        d,
        Decision::Divide {
            total: 4,
            min_each: 1,
            ..
        }
    )));
    assert_eq!(division_of(&t, top(&t)), vec![1, 2, 1]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(orni)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(in_graveyard(&t, bears));
    assert_eq!(t.obj_now(giant).damage, 1);
    // A division giving a target 0 isn't accepted: each target still gets at least 1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    flamefiend_enters(&mut t, &[pl(P1), obj(bears)], &[4, 0]);
    let d = division_of(&t, top(&t));
    assert_eq!(d.len(), 2);
    assert!(
        d.iter().all(|n| *n >= 1) && d.iter().sum::<u32>() == 4,
        "{d:?}"
    );
}

#[test]
fn kuldotha_flamefiend_chooses_its_sacrifice_on_resolution() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Kuldotha Flamefiend",
        "You don't choose whether to sacrifice an artifact or which artifact you're sacrificing until the ability resolves. Once you decide to sacrifice an artifact (or not), it's too late for players to respond."
    );
    let mut t = TestGame::new(2);
    let from = mark(&t);
    let orni = flamefiend_enters(&mut t, &[pl(P1)], &[4]);
    assert_eq!(yes_no_since(&t, P0, from), 0, "not chosen yet");
    assert!(t.on_battlefield(orni));
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(orni)]);
    t.g.resolve_top();
    // Sacrificed and the damage dealt in one resolution, with no priority in between.
    assert_eq!(yes_no_since(&t, P0, from), 1);
    assert!(in_graveyard(&t, orni));
    assert_eq!(t.life(P1), 16);
    assert!(priority_since(&t, from).is_empty());
}

#[test]
fn kuldotha_flamefiend_with_illegal_targets() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Kuldotha Flamefiend",
        "If some of the targets become illegal for Kuldotha Flamefiend's ability, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn't dealt at all. If all of the targets become illegal, you can't sacrifice an artifact."
    );
    // Some illegal: the Bears' 1 isn't dealt to anyone else.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let orni = flamefiend_enters(&mut t, &[pl(P1), obj(bears), obj(giant)], &[2, 1, 1]);
    bounce(&mut t, bears);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(orni)]);
    t.resolve_all();
    assert!(in_graveyard(&t, orni));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(giant).damage, 1);
    // All illegal: the ability doesn't resolve; nothing is sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let orni = flamefiend_enters(&mut t, &[obj(bears)], &[4]);
    bounce(&mut t, bears);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(orni)]);
    t.resolve_all();
    assert_eq!(yes_no_since(&t, P0, from), 0);
    assert!(t.on_battlefield(orni));
}

#[test]
fn last_ditch_effort_sacrifices_on_resolution_one_damage_each() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Last-Ditch Effort",
        "The creatures are chosen and sacrificed on resolution."
    );
    ruling!(
        "Last-Ditch Effort",
        "It deals one damage for each creature sacrificed."
    );
    supported("Last-Ditch Effort");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Last-Ditch Effort");
    let from = mark(&t);
    t.cast(P0, spell).target(pl(P1)).go();
    // Nothing is chosen or sacrificed as it's cast.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })));
    assert!([a, b, c].iter().all(|id| t.on_battlefield(*id)));
    // In response, the Ornithopter leaves; it can't be chosen on resolution.
    bounce(&mut t, c);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.resolve_all();
    assert!(in_graveyard(&t, a) && in_graveyard(&t, b));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn each_opponent_in_two_headed_giant_is_the_whole_team() {
    cr!("810.9", "120.3a");
    ruling!(
        "Embraal Gear-Smasher",
        "In a Two-Headed Giant game, Embraal Gear-Smasher's ability causes 4 damage total to be dealt to the opposing team."
    );
    ruling!(
        "Drannith Stinger",
        "In a Two-Headed Giant game, Drannith Stinger’s first ability causes the opposing team to lose 2 life."
    );
    supported("Embraal Gear-Smasher");
    supported("Drannith Stinger");
    // Embraal Gear-Smasher: "{T}, Sacrifice an artifact: This creature deals 2 damage to
    // each opponent."
    let mut t = two_headed_giant();
    let smasher = t.battlefield(P0, "Embraal Gear-Smasher");
    let orni = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(orni)]);
    t.activate(P0, smasher, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (26, 26));
    assert_eq!((t.life(P0), t.life(P1)), (30, 30));
    // Drannith Stinger: "Whenever you cycle another card, this creature deals 1 damage to
    // each opponent."
    let mut t = two_headed_giant();
    t.battlefield(P0, "Drannith Stinger");
    t.lands(P0, "Wastes", 1);
    let other = t.hand(P0, "Drannith Stinger");
    cycle(&mut t, P0, other, 0).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (28, 28));
}

#[test]
fn susurian_voidborn_with_an_illegal_target_opponent_gains_no_life() {
    cr!("608.2b");
    ruling!(
        "Susurian Voidborn",
        "If the target opponent is an illegal target as Susurian Voidborn’s first ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain life."
    );
    supported("Susurian Voidborn");
    for hexproof in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Susurian Voidborn");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[pl(P1)]);
        kill(&mut t, bears);
        assert_eq!(t.stack_len(), 1);
        if hexproof {
            // In response, P1 gets hexproof (Leyline of Sanctity).
            t.battlefield(P1, "Leyline of Sanctity");
        }
        t.resolve_all();
        let (gain, loss) = if hexproof { (0, 0) } else { (1, 1) };
        assert_eq!((t.life(P0), t.life(P1)), (20 + gain, 20 - loss));
    }
}
