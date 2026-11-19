//! Rulings batch P086 — combat hate: evasion against a creature type or kind (Kor
//! Castigator, Rime Dryad, Ayumi, Cynical Loner, Pompous Gadabout, Temple Thief),
//! Diregraf Escort, Pyrophobia, Karlov Watchdog, Bloodvial Purveyor, Dinosaur Hunter,
//! Circle of Protection: Shadow and Plague Sliver.

use crate::r_p086_common::*;
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A 1/1 creature with the given creature types (a custom card) on `p`'s battlefield.
fn typed(t: &mut TestGame, p: PlayerId, types: &str) -> ObjectId {
    let def = crate::r_s01_common::custom_card(
        types,
        &format!("Creature — {types}"),
        "{1}",
        Some((1, 1)),
        "",
    );
    t.custom(p, def, Zone::Battlefield)
}

/// P0 attacks P1 with `attacker`; returns whether P1 may block it with `blocker`.
fn can_block(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) -> bool {
    if t.g.turn.step != Step::DeclareAttackers {
        attack_with(t, &[(attacker, Entity::Player(P1))]);
    }
    legal_blocks(t, P1, &[(blocker, attacker)])
}

#[test]
fn evasion_against_a_type_or_kind_cases() {
    cr!("509.1b", "702.14c", "205.3");
    ruling!(
        "Kor Castigator",
        "A creature that’s an Eldrazi but not a Scion (or vice versa) can block Kor Castigator."
    );
    ruling!("Rime Dryad", "Can’t landwalk through a non-Snow Forest.");
    ruling!(
        "Ayumi, the Last Visitor",
        "Nonland legendary permanents don’t affect whether Ayumi can be blocked or not, only legendary lands."
    );
    for name in ["Kor Castigator", "Rime Dryad", "Ayumi, the Last Visitor"] {
        supported(name);
    }
    // Kor Castigator.
    for (types, ok) in [("Eldrazi", true), ("Scion", true), ("Eldrazi Scion", false)] {
        let mut t = TestGame::new(2);
        let kor = t.battlefield(P0, "Kor Castigator");
        let b = typed(&mut t, P1, types);
        assert_eq!(can_block(&mut t, kor, b), ok, "{types}");
    }
    // Rime Dryad.
    for (land, ok) in [("Forest", true), ("Snow-Covered Forest", false)] {
        let mut t = TestGame::new(2);
        let dryad = t.battlefield(P0, "Rime Dryad");
        t.battlefield(P1, land);
        let bears = t.battlefield(P1, "Grizzly Bears");
        assert_eq!(can_block(&mut t, dryad, bears), ok, "{land}");
    }
    // Ayumi.
    for (perm, ok) in [("Avacyn, Angel of Hope", true), ("Gaea's Cradle", false)] {
        let mut t = TestGame::new(2);
        let ayumi = t.battlefield(P0, "Ayumi, the Last Visitor");
        t.battlefield(P1, perm);
        let bears = t.battlefield(P1, "Grizzly Bears");
        assert_eq!(can_block(&mut t, ayumi, bears), ok, "{perm}");
    }
}

#[test]
fn blocked_stays_blocked_when_the_blocker_stops_qualifying() {
    cr!("509.1h", "506.4");
    ruling!(
        "Cynical Loner",
        "Once Cynical Loner has been blocked by a creature, giving that creature the Glimmer creature type won't cause Cynical Loner to become unblocked."
    );
    ruling!(
        "Pompous Gadabout",
        "Once Pompous Gadabout has been blocked by a creature, turning that creature face down or otherwise causing it to lose its name won't cause it to stop blocking Pompous Gadabout."
    );
    ruling!(
        "Temple Thief",
        "Once a creature has blocked Temple Thief, causing that creature to become enchanted (or, in some rare cases, causing it to become an enchantment creature) won’t cause Temple Thief to become unblocked."
    );
    for name in ["Cynical Loner", "Pompous Gadabout", "Temple Thief"] {
        supported(name);
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let changeling = t.battlefield(P1, "Amoeboid Changeling");
        crate::r_s03_common::to_blockers(
            &mut t,
            &[(attacker, Entity::Player(P1))],
            &[(bears, attacker)],
        );
        match name {
            "Cynical Loner" => {
                // Amoeboid Changeling: "Target creature gains all creature types".
                activate(
                    &mut t,
                    P1,
                    changeling,
                    "gains all creature types",
                    &[obj(bears)],
                );
                t.resolve_all();
                assert!(t.obj_now(bears).chars.has_subtype("Glimmer"));
            }
            "Pompous Gadabout" => {
                assert!(mtg_engine::facedown::turn_face_down(&mut t.g, bears));
                t.g.recompute();
                assert!(t.obj_now(bears).face_down);
            }
            _ => {
                attach_new(&mut t, P1, "Guildscorn Ward", obj(bears));
            }
        }
        t.settle();
        assert!(
            crate::r_s21_common::blocks_now(&t).contains(&(t.g.current(bears), attacker)),
            "{name}"
        );
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}");
    }
}

#[test]
fn diregraf_escort_can_pair_with_a_zombie() {
    cr!("702.95a", "702.16");
    ruling!(
        "Diregraf Escort",
        "Diregraf Escort can be paired with a Zombie. The protection will have no effect on the pairing."
    );
    supported("Diregraf Escort");
    let mut t = TestGame::new(2);
    let zombie = t.battlefield(P0, "Walking Corpse");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(zombie)]);
    let escort = t.enter(P0, "Diregraf Escort");
    t.resolve_all();
    assert_eq!(t.obj_now(escort).paired_with, Some(zombie));
    assert!(t.obj_now(zombie).paired_with.is_some());
    assert!(t.on_battlefield(zombie));
    // A Zombie can't block either of them.
    let other = t.battlefield(P1, "Walking Corpse");
    assert!(!can_block(&mut t, zombie, other));
    assert_eq!(t.obj_now(escort).paired_with, Some(zombie));
}

// --- Pyrophobia -------------------------------------------------------------------------------

#[test]
fn pyrophobia_affects_creatures_that_become_cowards_later() {
    cr!("611.2c", "702.73a");
    ruling!(
        "Pyrophobia",
        "Because it doesn't modify the characteristics of creatures, Pyrophobia can affect Cowards that enter the battlefield or creatures that become Cowards after the spell resolves."
    );
    supported("Pyrophobia");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let target = t.battlefield(P1, "Craw Wurm");
    let changeling = t.battlefield(P1, "Amoeboid Changeling");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Pyrophobia", &[obj(target)]);
    t.resolve_all();
    // Woodland Changeling (changeling: a Coward) enters later.
    let late = t.enter(P1, "Woodland Changeling");
    // Grizzly Bears becomes a Coward later.
    activate(
        &mut t,
        P1,
        changeling,
        "gains all creature types",
        &[obj(bears)],
    );
    t.resolve_all();
    assert!(!can_block(&mut t, goblin, late));
    assert!(!legal_blocks(&mut t, P1, &[(bears, goblin)]));
}

#[test]
fn pyrophobia_with_an_illegal_target_lets_cowards_block() {
    cr!("608.2b");
    ruling!(
        "Pyrophobia",
        "If the target creature is an illegal target by the time Pyrophobia tries to resolve, the spell doesn't resolve. Cowards can block if they're otherwise allowed to."
    );
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let target = t.battlefield(P1, "Craw Wurm");
    let coward = t.battlefield(P1, "Woodland Changeling");
    cast_new(&mut t, P0, "Pyrophobia", &[obj(target)]);
    destroy(&mut t, target);
    t.resolve_all();
    assert!(can_block(&mut t, goblin, coward));
}

// --- Karlov Watchdog, Bloodvial Purveyor --------------------------------------------------------

#[test]
fn karlov_watchdog_pumps_whatever_happens_to_the_attackers() {
    cr!("603.2", "508.1m", "611.2c");
    ruling!(
        "Karlov Watchdog",
        "It doesn't matter what happens to the creatures in response. As long as you attacked with at least three creatures, creatures you control will get +1/+1 until end of turn when Karlov Watchdog's triggered ability resolves."
    );
    supported("Karlov Watchdog");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Karlov Watchdog");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Craw Wurm");
    attack_with(
        &mut t,
        &[
            (dog, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.pt(dog), (4, 3));
    assert_eq!(t.pt(home), (7, 5));
}

#[test]
fn bloodvial_purveyor_counts_the_defending_players_blood_only() {
    cr!("506.2", "508.5");
    ruling!(
        "Bloodvial Purveyor",
        "Only Blood tokens controlled by the player Bloodvial Purveyor is attacking or the controller of the planeswalker it's attacking are counted to determine the bonus from its last ability."
    );
    // Attacking P1 counts P1's two; attacking P2's planeswalker, P2's three.
    for (to_p2, power) in [(false, 7), (true, 8)] {
        let mut t = TestGame::new(3);
        let purveyor = t.battlefield(P0, "Bloodvial Purveyor");
        for (p, n) in [(P0, 1), (P1, 2), (P2, 3)] {
            for _ in 0..n {
                create_token(&mut t, p, "Blood");
            }
        }
        let gideon = t.battlefield(P2, "Gideon, Ally of Zendikar");
        let to = if to_p2 {
            obj(gideon)
        } else {
            Entity::Player(P1)
        };
        attack_with(&mut t, &[(purveyor, to)]);
        t.resolve_all();
        assert_eq!(t.pt(purveyor), (power, 6), "to_p2={to_p2}");
    }
}

// --- Dinosaur Hunter --------------------------------------------------------------------------

#[test]
fn dinosaur_hunter_and_enrage_resolve_in_apnap_order() {
    cr!("101.4", "603.3b");
    ruling!(
        "Dinosaur Hunter",
        "If an opponent’s Dinosaur has an enrage ability and your Dinosaur Hunter deals damage to it during your turn, that ability resolves before Dinosaur Hunter’s ability destroys the Dinosaur. If it’s that opponent’s turn, the Dinosaur is destroyed first, but its enrage ability still resolves afterwards."
    );
    // Ripjaw Raptor: "Enrage — Whenever this creature is dealt damage, draw a card."
    // P0's turn: the Hunter attacks and the Raptor blocks.
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Dinosaur Hunter");
    let raptor = t.battlefield(P1, "Ripjaw Raptor");
    let hand = t.hand_size(P1);
    attack_with(&mut t, &[(hunter, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(raptor, hunter)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.on_battlefield(raptor));
    t.resolve_all();
    assert!(!t.on_battlefield(raptor));
    // P1's turn: the Raptor attacks and the Hunter blocks.
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Dinosaur Hunter");
    let raptor = t.battlefield(P1, "Ripjaw Raptor");
    t.set_step(P1, Step::BeginningOfCombat);
    let hand = t.hand_size(P1);
    attack_with(&mut t, &[(raptor, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(hunter, raptor)]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(!t.on_battlefield(raptor));
    assert_eq!(t.hand_size(P1), hand);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
}

// --- Circle of Protection: Shadow, Plague Sliver -------------------------------------------------

#[test]
fn circle_of_protection_shadow_prevents_noncombat_damage_too() {
    cr!("615.1a", "702.28a");
    ruling!(
        "Circle of Protection: Shadow",
        "It can be used on non-combat damage which happens to come from creatures with the Shadow ability."
    );
    supported("Circle of Protection: Shadow");
    let mut t = TestGame::new(2);
    let cop = t.battlefield(P0, "Circle of Protection: Shadow");
    let slayer = t.battlefield(P1, "Dauthi Slayer");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(slayer)]);
    activate(&mut t, P0, cop, "prevent", &[]);
    t.resolve_all();
    run_effect(
        &mut t,
        Some(slayer),
        P1,
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(2),
            to: Sel::Target(0),
        },
        &[Entity::Player(P0)],
    );
    assert_eq!(t.life(P0), 20);
}

#[test]
fn plague_sliver_each_sliver_damages_its_own_controller() {
    cr!("603.3a", "109.5");
    ruling!(
        "Plague Sliver",
        "Each Sliver deals damage to its controller (not to Plague Sliver’s controller)."
    );
    supported("Plague Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plague Sliver");
    t.battlefield(P1, "Muscle Sliver");
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
}
