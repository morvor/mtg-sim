//! CR 510: the combat damage step.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assign(t: &mut TestGame, p: PlayerId, v: &[i64]) {
    t.answer(p, DecisionKind::Damage, Answer::Numbers(v.to_vec()));
}

fn grant(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    apply(
        t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(kw))],
            duration: Duration::EndOfTurn,
        },
        &[id],
    );
}

fn strip(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    apply(
        t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        &[id],
    );
}

#[test]
fn attacking_player_assigns_first_then_defending_player() {
    cr!("510.1");
    let mut t = TestGame::new(2);
    let big = t.battlefield(P0, "Craw Wurm");
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let a3 = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Hill Giant");
    let y = t.battlefield(P1, "Grizzly Bears");
    let guard = bf(
        &mut t,
        P1,
        custom_card(
            "Double Guard",
            "Creature — Test",
            Some((4, 6)),
            "This creature can block an additional creature each combat.",
        ),
    );
    declare(
        &mut t,
        &[
            (big, Entity::Player(P1)),
            (a2, Entity::Player(P1)),
            (a3, Entity::Player(P1)),
        ],
    );
    block(&mut t, P1, &[(x, big), (y, big), (guard, a2), (guard, a3)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::CombatDamage);
    let order: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::AssignCombatDamage { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    // Assigning and dealing combat damage didn't use the stack.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn creatures_assign_damage_equal_to_power_and_nothing_if_zero_or_less() {
    cr!("510.1a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let zero = bf(&mut t, P0, vanilla("Paper Wall", 0, 2));
    let weak = bf(&mut t, P0, vanilla("Weakling", 1, 3));
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(-3), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        &[weak],
    );
    assert_eq!(t.pt(weak), (-2, 3));
    declare(
        &mut t,
        &[
            (giant, Entity::Player(P1)),
            (zero, Entity::Player(P1)),
            (weak, Entity::Player(P1)),
        ],
    );
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    let sources: Vec<ObjectId> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, .. } => Some(*source),
                _ => None,
            })
            .collect();
    assert_eq!(sources, vec![giant]);
}

#[test]
fn damage_is_divided_freely_among_blockers() {
    cr!("510.1c");
    // CR 510.1c example: Elvish Regrower (4/3) blocked by Vampire Spawn (2/3) and Helpful
    // Hunter (1/1); its controller may assign all 4 damage to the Hunter.
    let mut t = TestGame::new(2);
    let regrower = t.battlefield(P0, "Elvish Regrower");
    let spawn = t.battlefield(P1, "Vampire Spawn");
    let hunter = t.battlefield(P1, "Helpful Hunter");
    declare(&mut t, &[(regrower, Entity::Player(P1))]);
    block(&mut t, P1, &[(spawn, regrower), (hunter, regrower)]);
    assign(&mut t, P0, &[0, 4]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(hunter));
    assert!(t.on_battlefield(spawn));
    assert_eq!(t.obj_now(spawn).damage, 0);

    // Or 3 to the Spawn and 1 to the Hunter.
    let mut t = TestGame::new(2);
    let regrower = t.battlefield(P0, "Elvish Regrower");
    let spawn = t.battlefield(P1, "Vampire Spawn");
    let hunter = t.battlefield(P1, "Helpful Hunter");
    declare(&mut t, &[(regrower, Entity::Player(P1))]);
    block(&mut t, P1, &[(spawn, regrower), (hunter, regrower)]);
    assign(&mut t, P0, &[3, 1]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(hunter));
    assert!(!t.on_battlefield(spawn));
}

#[test]
fn blocker_divides_damage_among_attackers_it_blocks() {
    cr!("510.1d");
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let guard = bf(
        &mut t,
        P1,
        custom_card(
            "Double Guard",
            "Creature — Test",
            Some((4, 6)),
            "This creature can block an additional creature each combat.",
        ),
    );
    declare(
        &mut t,
        &[(a1, Entity::Player(P1)), (a2, Entity::Player(P1))],
    );
    block(&mut t, P1, &[(guard, a1), (guard, a2)]);
    assign(&mut t, P1, &[0, 4]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(a1));
    assert_eq!(t.obj_now(a1).damage, 0);
    assert!(!t.on_battlefield(a2));
}

#[test]
fn illegal_damage_assignment_is_undone() {
    cr!("510.1e");
    let mut t = TestGame::new(2);
    let regrower = t.battlefield(P0, "Elvish Regrower");
    let spawn = t.battlefield(P1, "Vampire Spawn");
    let hunter = t.battlefield(P1, "Helpful Hunter");
    declare(&mut t, &[(regrower, Entity::Player(P1))]);
    block(&mut t, P1, &[(spawn, regrower), (hunter, regrower)]);
    // 5 damage from a 4-power creature is illegal; the engine assigns legally instead.
    assign(&mut t, P0, &[4, 1]);
    go_to(&mut t, Step::EndOfCombat);
    let total: u32 =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, amount, .. } if *source == regrower => Some(*amount),
                _ => None,
            })
            .sum();
    assert_eq!(total, 4);
}

#[test]
fn combat_damage_is_dealt_simultaneously() {
    cr!("510.2");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Hill Giant");
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block(&mut t, P1, &[(x, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    // Both giants were destroyed: each dealt damage though the other's damage was lethal.
    assert!(!t.on_battlefield(a) && !t.on_battlefield(x));
    assert_eq!(t.life(P1), 18);
    // All combat damage events happened together in one action: nothing but damage and
    // its results (life loss) happened between the first and the last one.
    let ev = &t.g.turn_events;
    let dmg: Vec<usize> = ev
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, Event::Damage { combat: true, .. }))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(dmg.len(), 3);
    assert!(ev[dmg[0]..=dmg[2]]
        .iter()
        .all(|e| matches!(e, Event::Damage { .. } | Event::LifeLost { .. })));
}

#[test]
fn active_player_gets_priority_in_the_combat_damage_step() {
    cr!("510.3");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    let order = priority_order_in(&mut t, Step::CombatDamage);
    assert_eq!(order, vec![P0, P1]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn damage_and_state_based_action_triggers_are_stacked_before_priority() {
    cr!("510.3a");
    let mut t = TestGame::new(2);
    let a = bf(
        &mut t,
        P0,
        custom_card(
            "Looter",
            "Creature — Test",
            Some((2, 2)),
            "Whenever this creature deals combat damage to a player, you gain 1 life.",
        ),
    );
    let b = t.battlefield(P0, "Grizzly Bears");
    let x = bf(
        &mut t,
        P1,
        custom_card(
            "Martyr",
            "Creature — Test",
            Some((1, 1)),
            "When this creature dies, you gain 1 life.",
        ),
    );
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block(&mut t, P1, &[(x, b)]);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.stack_len(), 0);
    t.script.lock().unwrap().asked.clear();
    t.g.advance();
    assert!(matches!(
        t.asked().first(),
        Some((P0, Decision::Priority { .. }))
    ));
    // The damage trigger and the dies trigger (from the SBA) are both on the stack.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn first_strike_creatures_deal_damage_first() {
    cr!("510.4");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[(knight, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, knight)]);
    go_to(&mut t, Step::FirstStrikeDamage);
    t.settle();
    assert!(!t.on_battlefield(bears), "killed in the first-strike step");
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(knight), "the bears never dealt damage");
}

#[test]
fn double_strike_deals_damage_in_both_steps() {
    cr!("510.4");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Boros Swiftblade");
    declare(&mut t, &[(blade, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 19);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn second_step_participants_are_fixed_as_the_first_step_began() {
    cr!("510.4");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Boros Swiftblade");
    let fireborn = t.battlefield(P0, "Fireborn Knight");
    declare(
        &mut t,
        &[
            (knight, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (blade, Entity::Player(P1)),
            (fireborn, Entity::Player(P1)),
        ],
    );
    go_to(&mut t, Step::FirstStrikeDamage);
    // First-strike step: knight 2, swiftblade 1, fireborn 2.
    assert_eq!(t.life(P1), 15);
    // Between steps: the bears gain first strike (they had neither as the first step
    // began, so they still deal damage), the knight gains double strike (it deals damage
    // again), and Fireborn Knight loses double strike (it doesn't).
    grant(&mut t, bears, KeywordKind::FirstStrike);
    grant(&mut t, knight, KeywordKind::DoubleStrike);
    strip(&mut t, fireborn, KeywordKind::DoubleStrike);
    go_to(&mut t, Step::CombatDamage);
    // Second step: bears 2, knight 2, swiftblade 1.
    assert_eq!(t.life(P1), 10);
}
