//! Rulings batch P075 — abilities that put -1/-1 counters: one counter regardless of the
//! dying creature's counters (Blowfly Infestation), a "may pay" paid once (Ruthless Sniper),
//! "unless" choices (Torment of Venom), wither on noncombat damage (Massacre Girl, Barbed
//! Bloodletter), distinct targets and divisions (Grind, Splendid Agony), a die rolled on
//! resolution (Six-Sided Die) and proliferating the creature just given a counter (Grim
//! Affliction).

use crate::r_p075_common::*;
use crate::r_s04_common::cycle;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn blowfly_infestation_puts_a_single_counter() {
    cr!("603.4", "603.10a", "122.1");
    ruling!(
        "Blowfly Infestation",
        "it will put a single -1/-1 counter on the targeted creature. It doesn't matter how many -1/-1 counters"
    );
    supported("Blowfly Infestation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blowfly Infestation");
    let giant = t.battlefield(P1, "Hill Giant");
    let victim = t.battlefield(P1, "Craw Wurm");
    put_counters(&mut t, giant, "-1/-1", 2);
    t.answer_targets(P0, &[obj(victim)]);
    destroy(&mut t, giant);
    assert_eq!(triggers_on_stack(&t, "-1/-1"), 1);
    t.resolve_all();
    assert_eq!(minus(&t, victim), 1);
    // A creature that dies with no -1/-1 counter doesn't trigger it.
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "-1/-1"), 0);
}

#[test]
fn ruthless_sniper_pays_only_once() {
    cr!("603.5", "603.12");
    ruling!(
        "Ruthless Sniper",
        "you can’t pay {1} multiple times to put more counters on creatures"
    );
    supported("Ruthless Sniper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruthless Sniper");
    let victim = t.battlefield(P1, "Craw Wurm");
    let land = t.hand(P0, "Lonely Sandbar");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[obj(victim)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    cycle(&mut t, P0, land, 0).expect("cycles");
    t.resolve_all();
    assert_eq!(minus(&t, victim), 1);
    assert_eq!(count_since(&t, from, is_yes_no), 1, "one payment offered");
    assert_eq!(pool_total(&t, P0), 2, "{{1}} paid once");
}

#[test]
fn torment_of_venom_controller_may_always_lose_life() {
    cr!("118.12", "608.2d");
    ruling!(
        "Torment of Venom",
        "That player can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    supported("Torment of Venom");
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Craw Wurm");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let hand_before = t.hand_size(P1);
    let from = t.asked().len();
    // P1 declines to sacrifice or discard.
    t.answer_yes(P1, false);
    t.answer_yes(P1, false);
    cast_card(&mut t, P0, "Torment of Venom", &[obj(victim)]);
    t.resolve_all();
    assert_eq!(minus(&t, victim), 3);
    assert!(t.on_battlefield(other));
    assert_eq!(t.hand_size(P1), hand_before);
    assert_eq!(t.life(P1), 17);
    assert!(
        asked_since(&t, from).iter().any(|(p, _)| *p == P1),
        "the creature's controller chooses"
    );
}

#[test]
fn wither_applies_to_noncombat_damage_dealt_by_creatures() {
    cr!("702.80a", "701.14a");
    ruling!(
        "Massacre Girl, Known Killer",
        "Wither applies to any damage dealt to creatures by creatures you control. This includes combat damage as well as anything that causes creatures you control to deal noncombat damage"
    );
    ruling!(
        "Barbed Bloodletter",
        "Wither applies to any damage dealt to creatures by the affected creature. This includes combat damage as well as anything that causes that creature to deal noncombat damage"
    );
    supported("Massacre Girl, Known Killer");
    supported("Barbed Bloodletter");
    for with_bloodletter in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spider = t.battlefield(P1, "Giant Spider");
        let power = if with_bloodletter {
            cast_card(&mut t, P0, "Barbed Bloodletter", &[obj(bears)]);
            t.resolve_all();
            3
        } else {
            t.battlefield(P0, "Massacre Girl, Known Killer");
            2
        };
        // Prey Upon: Grizzly Bears fights Giant Spider (noncombat damage).
        cast_card(&mut t, P0, "Prey Upon", &[obj(bears), obj(spider)]);
        t.resolve_all();
        assert_eq!(minus(&t, spider), power, "bloodletter={with_bloodletter}");
        assert_eq!(t.obj_now(spider).damage, 0, "no damage marked");
    }
    // Combat damage too (Massacre Girl).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Massacre Girl, Known Killer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    block_and_finish(&mut t, P1, &[(wurm, bears)]);
    assert_eq!(minus(&t, wurm), 2);
}

#[test]
fn grind_cant_target_the_same_creature_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Grind // Dust",
        "You can't target the same creature twice with Grind to give it two -1/-1 counters."
    );
    supported("Grind // Dust");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    let c = t.hand(P0, "Grind // Dust");
    t.answer_targets(P0, &[obj(wurm), obj(wurm)]);
    // Naming it twice is an invalid answer.
    match t.cast(P0, c).method(CastMethod::Half(0)).try_go() {
        Ok(spell) => {
            let chosen: Vec<Entity> = t
                .obj(spell)
                .stack
                .as_ref()
                .unwrap()
                .chosen
                .iter()
                .flat_map(|cm| cm.targets.iter().flatten().copied())
                .collect();
            // The invalid answer is rejected (falling back to a legal choice): the
            // creature is never chosen for both instances of "target".
            let times = chosen.iter().filter(|e| **e == obj(wurm)).count() as u32;
            assert!(times <= 1, "{chosen:?}");
            t.resolve_all();
            assert_eq!(minus(&t, wurm), times, "one counter per time it's a target");
        }
        Err(_) => assert_eq!(minus(&t, wurm), 0),
    }
    // Two different creatures: one counter each.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let c = t.hand(P0, "Grind // Dust");
    t.answer_targets(P0, &[obj(wurm), obj(giant)]);
    t.cast(P0, c).method(CastMethod::Half(0)).go();
    t.resolve_all();
    assert_eq!((minus(&t, wurm), minus(&t, giant)), (1, 1));
}

#[test]
fn splendid_agony_divides_as_cast_at_least_one_each() {
    cr!("601.2d", "115.7");
    ruling!(
        "Splendid Agony",
        "You choose how the counters will be distributed as you cast Splendid Agony. Each target creature must be assigned at least one counter."
    );
    supported("Splendid Agony");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.answer_targets(P0, &[obj(a), obj(b)]);
    give_mana_for(&mut t, P0, "Splendid Agony");
    let c = t.hand(P0, "Splendid Agony");
    t.cast(P0, c).go();
    let min_each = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(min_each),
            _ => None,
        });
    assert_eq!(min_each, Some(1), "divided as it's cast, at least one each");
    t.resolve_all();
    assert_eq!((minus(&t, a), minus(&t, b)), (1, 1));
}

#[test]
fn six_sided_die_rolls_only_as_it_resolves() {
    cr!("601.2c", "706.2", "608.2");
    ruling!(
        "Six-Sided Die",
        "You choose the target creature as you cast Six-Sided Die. You don't roll the die until the spell begins resolving."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.dice.loaded.push_back(2);
    cast_card(&mut t, P0, "Six-Sided Die", &[obj(wurm)]);
    assert_eq!(t.g.dice.loaded.len(), 1, "not rolled while on the stack");
    t.resolve_all();
    assert!(t.g.dice.loaded.is_empty());
    assert_eq!(minus(&t, wurm), 2);
}

#[test]
fn grim_affliction_proliferates_the_same_creature() {
    cr!("701.34a");
    ruling!(
        "Grim Affliction",
        "You may choose the creature you put the -1/-1 counter on to get another -1/-1 counter when proliferating."
    );
    supported("Grim Affliction");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(wurm)]);
    cast_card(&mut t, P0, "Grim Affliction", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(minus(&t, wurm), 2);
}
