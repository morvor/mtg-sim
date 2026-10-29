//! Rulings batch P006 — sacrificing for damage: sacrifice-a-creature additional costs
//! (CR 601.2b, 601.2h), sacrificing the source to pay its own ability's cost (CR 602.2c,
//! 113.7a), divided damage fixed on activation (CR 601.2d, 608.2b), abilities with an
//! illegal target (CR 608.2b), "sacrifice this; if you do" triggers that trigger several
//! times (CR 608.2c), and the timing of damage and state-based actions (CR 510.2, 704.3).

use crate::r_p006_common::*;
use crate::r_p109_common::{divide, division_of};
use crate::r_s01_common::{attack_with, block_and_finish, supported, watch};
use crate::r_s02_common::can_cast;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn sacrifice_a_creature_cost_is_exactly_one() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Altar's Reap",
        "You must sacrifice exactly one creature to cast this spell; you cannot cast it without sacrificing a creature, and you cannot sacrifice additional creatures."
    );
    // No creature: can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let reap = t.hand(P0, "Altar's Reap");
    assert!(!can_cast(&mut t, P0, reap, CastMethod::Normal));
    assert!(t.cast(P0, reap).try_go().is_err());
    // Two creatures: exactly one is sacrificed, even when asked for both.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let reap = t.hand(P0, "Altar's Reap");
    assert!(can_cast(&mut t, P0, reap, CastMethod::Normal));
    let from = mark(&t);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    let _ = t.cast(P0, reap).try_go();
    let c = choices_since(&t, P0, from);
    assert!(!c.is_empty());
    assert_eq!((c[0].1, c[0].2), (1, 1), "exactly one");
    let gone = [a, b].iter().filter(|id| !t.on_battlefield(**id)).count();
    assert_eq!(gone, 1);
}

#[test]
fn kuldotha_flamefiend_with_zero_targets_still_sacrifices() {
    cr!("601.2c", "603.3d", "608.2c");
    ruling!(
        "Kuldotha Flamefiend",
        "You may choose zero targets. If you do, Kuldotha Flamefiend deals no damage, but you may sacrifice an artifact."
    );
    supported("Kuldotha Flamefiend");
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    let fiend = t.enter(P0, "Kuldotha Flamefiend");
    t.settle();
    assert_eq!(triggers_from(&t, fiend), 1);
    assert!(targets_of(&t, top(&t)).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(orni)]);
    t.resolve_all();
    assert!(in_graveyard(&t, orni), "sacrificed");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn flame_kin_war_scout_triggers_twice_but_sacrifices_once() {
    cr!("603.2", "608.2c", "701.21a");
    ruling!(
        "Flame-Kin War Scout",
        "If multiple creatures enter at the same time, Flame-Kin War Scout's ability will trigger multiple times."
    );
    supported("Flame-Kin War Scout");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Flame-Kin War Scout");
    let giants = crate::r_p116_common::enter_together(&mut t, P1, &["Hill Giant", "Hill Giant"]);
    t.settle();
    assert_eq!(triggers_from(&t, scout), 2);
    t.resolve();
    assert!(in_graveyard(&t, scout));
    let dead = giants.iter().filter(|g| in_graveyard(&t, **g)).count();
    assert_eq!(dead, 1, "the first trigger's creature takes 4");
    t.resolve_all();
    let alive: Vec<_> = giants.iter().filter(|g| t.on_battlefield(**g)).collect();
    assert_eq!(alive.len(), 1);
    assert_eq!(t.obj_now(*alive[0]).damage, 0, "no damage from the second trigger");
}

/// Activates Mogg Mob's ability dividing `division` among `targets`.
fn mogg_mob(t: &mut TestGame, targets: &[Entity], division: &[i64]) -> Result<(), String> {
    let mob = t.battlefield(P0, "Mogg Mob");
    t.answer_targets(P0, targets);
    divide(t, P0, division);
    t.answer_choose(P0, &[obj(mob)]);
    let r = t.activate(P0, mob, 0, &[]);
    t.clear_answers();
    r.map(|_| ()).map_err(|e| format!("{e:?}"))
}

#[test]
fn mogg_mob_divides_on_activation() {
    cr!("601.2d", "602.2b");
    ruling!(
        "Mogg Mob",
        "You choose how many targets Mogg Mob's ability has and how the damage is divided as you activate the ability. Each target must receive at least 1 damage."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    let from = mark(&t);
    mogg_mob(&mut t, &[obj(a), obj(b)], &[2, 1]).unwrap();
    let divides: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(*min_each),
            _ => None,
        })
        .collect();
    assert_eq!(divides, vec![1], "asked on activation, at least 1 each");
    assert_eq!(division_of(&t, top(&t)), vec![2, 1]);
    t.resolve_all();
    assert_eq!((t.obj_now(a).damage, t.obj_now(b).damage), (2, 1));
    // One target: all 3 to it.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    mogg_mob(&mut t, &[obj(a)], &[3]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 3);
}

#[test]
fn mogg_mob_loses_the_damage_of_an_illegal_target() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Mogg Mob",
        "If some of the targets are illegal as Mogg Mob's activated ability tries to resolve, the original division of damage still applies and the damage that would have been dealt to the illegal targets is lost."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Grizzly Bears");
    mogg_mob(&mut t, &[obj(a), obj(b), pl(P1)], &[1, 1, 1]).unwrap();
    crate::r_p046_common::bounce(&mut t, b);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1, "not 2");
    assert_eq!(t.life(P1), 19, "not 18");
}

#[test]
fn abilities_with_an_illegal_single_target_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Immersturm Skullcairn",
        "If the target player is an illegal target as the last activated ability tries to resolve, that ability won’t resolve and none of its effects will happen."
    );
    ruling!(
        "Grotag Siege-Runner",
        "If the targeted creature is an illegal target by the time the ability resolves, the ability doesn't resolve. Its controller won't be dealt damage."
    );
    supported("Immersturm Skullcairn");
    supported("Grotag Siege-Runner");
    // Skullcairn: P1 gets hexproof (Leyline of Sanctity) in response.
    let mut t = TestGame::new(2);
    let cairn = t.battlefield(P0, "Immersturm Skullcairn");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    t.hand(P1, "Grizzly Bears");
    t.activate(P0, cairn, 1, &[pl(P1)]).unwrap();
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 1, "no discard");
    // Grotag Siege-Runner: the Wall is returned to hand in response.
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Grotag Siege-Runner");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[obj(runner)]);
    t.activate(P0, runner, 0, &[obj(wall)]).unwrap();
    crate::r_p046_common::bounce(&mut t, wall);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Legal: destroyed, and its controller is dealt 2.
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Grotag Siege-Runner");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[obj(runner)]);
    t.activate(P0, runner, 0, &[obj(wall)]).unwrap();
    t.resolve_all();
    assert!(in_graveyard(&t, wall));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mogg_fanatic_is_destroyed_before_anyone_gets_priority() {
    cr!("510.2", "704.3", "117.3c");
    ruling!(
        "Mogg Fanatic",
        "Once Mogg Fanatic has dealt and been dealt combat damage, it’s too late to activate its ability before it’s destroyed."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let fanatic = t.battlefield(P1, "Mogg Fanatic");
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            (g.turn.step == Step::CombatDamage).then(|| {
                g.battlefield
                    .iter()
                    .any(|id| g.obj(*id).chars.name == "Mogg Fanatic")
            })
        },
    );
    attack_with(&mut t, &[(giant, pl(P1))]);
    block_and_finish(&mut t, P1, &[(fanatic, giant)]);
    let first = seen.lock().unwrap().iter().flatten().next().copied();
    assert_eq!(first, Some(false), "already in the graveyard");
    assert!(in_graveyard(&t, fanatic));
    assert_eq!(t.obj_now(giant).damage, 1);
}

#[test]
fn throwing_knife_kills_a_blocker_before_blocks() {
    cr!("508.1m", "508.2", "509.1");
    ruling!(
        "Throwing Knife",
        "Throwing Knife's triggered ability triggers before blockers are declared. If you use the ability and destroy the targeted creature with lethal damage, it won't be on the battlefield to block."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let knife = crate::r_s06_common::attach_new(&mut t, P0, "Throwing Knife", obj(giant));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(giant, pl(P1))]);
    assert_eq!(triggers_from(&t, knife), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(in_graveyard(&t, knife));
    assert!(in_graveyard(&t, bears));
    let from = mark(&t);
    t.advance_to(P0, Step::EndOfCombat);
    let options: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::DeclareBlockers { options } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(options.iter().all(|o| o.is_empty()), "no blockers left");
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sacrificing_the_source_to_its_own_ability() {
    cr!("602.2b", "601.2h", "113.7a", "608.2h");
    ruling!(
        "Arms Dealer",
        "You can sacrifice Arms Dealer to activate its own ability."
    );
    ruling!(
        "Magmaw",
        "You can sacrifice Magmaw to pay the cost for its own ability. It still deals 1 damage to the target."
    );
    supported("Arms Dealer");
    supported("Magmaw");
    let mut t = TestGame::new(2);
    let dealer = t.battlefield(P0, "Arms Dealer");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 2);
    t.answer_choose(P0, &[obj(dealer)]);
    t.activate(P0, dealer, 0, &[obj(wurm)]).unwrap();
    assert!(in_graveyard(&t, dealer));
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 4);

    let mut t = TestGame::new(2);
    let magmaw = t.battlefield(P0, "Magmaw");
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[obj(magmaw)]);
    t.activate(P0, magmaw, 0, &[pl(P1)]).unwrap();
    assert!(in_graveyard(&t, magmaw));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn fire_shrine_keeper_cant_target_one_creature_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Fire Shrine Keeper",
        "You can’t target the same creature twice to have Fire Shrine Keeper deal 6 damage to it."
    );
    supported("Fire Shrine Keeper");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Fire Shrine Keeper");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 8);
    t.answer_targets(P0, &[obj(wurm), obj(wurm)]);
    let r = t.activate(P0, keeper, 0, &[]);
    t.clear_answers();
    match r {
        // The duplicate is rejected outright...
        Err(_) => assert!(t.on_battlefield(keeper)),
        // ...or only one target is kept.
        Ok(_) => assert!(targets_of(&t, top(&t)).len() <= 1),
    }
    t.resolve_all();
    assert!(t.on_battlefield(wurm), "6/4 dealt at most 3");
    assert!(t.obj_now(wurm).damage <= 3);
    // Two different creatures: 3 each.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Fire Shrine Keeper");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 8);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer_choose(P0, &[obj(keeper)]);
    t.activate(P0, keeper, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.obj_now(a).damage, t.obj_now(b).damage), (3, 3));
}
