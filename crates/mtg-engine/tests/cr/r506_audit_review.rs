//! Review of the combat audit (CR 506–511). Requirements on large boards (CR 508.1d,
//! 509.1c): the maximum number of requirements obeyed is found whether the creatures are
//! alike (tokens: the search tries each way of splitting alike creatures once) or all
//! different (too many declarations to try: the search starts from greedy declarations
//! that need no cost and break no restriction); and a few damage and blocking
//! interactions the audit didn't reach.

use crate::r506_audit_common::*;
use crate::r506_common::{bf, block, declare, go_to, vanilla};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `n` creatures for `p`: Grizzly Bears, or (`different`) 2/2 creatures with different
/// names, which the search can't treat as interchangeable.
fn creatures(t: &mut TestGame, p: PlayerId, n: usize, different: bool) -> Vec<ObjectId> {
    (0..n)
        .map(|i| {
            if different {
                bf(t, p, vanilla(&format!("Test Soldier {i}"), 2, 2))
            } else {
                t.battlefield(p, "Grizzly Bears")
            }
        })
        .collect()
}

#[test]
fn creatures_that_must_block_on_a_large_board_block_where_no_cost_is_needed() {
    cr!("509.1c", "509.1d");
    interp!("requirement-search-budget");
    supported("Awesome Presence");
    supported("Grand Melee");
    for different in [false, true] {
        let mut t = TestGame::new(2);
        // Grand Melee: all creatures attack and block each combat if able. Awesome
        // Presence on the Giant: "Enchanted creature can't be blocked unless defending
        // player pays {3} for each creature they control that's blocking it."
        t.battlefield(P0, "Grand Melee");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let presence = t.battlefield(P0, "Awesome Presence");
        assert!(t.g.attach(presence, Entity::Object(giant)));
        let mine = creatures(&mut t, P1, 17, different);
        t.set_step(P0, Step::BeginningOfCombat);
        declare(
            &mut t,
            &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        // P1 has no mana: each of its creatures blocks the Bears, which costs nothing
        // (blocking the Giant would need {3} each, which P1 isn't required to pay).
        go_to(&mut t, Step::DeclareBlockers);
        assert!(blockers_of(&t, giant).is_empty(), "different: {different}");
        assert_eq!(blockers_of(&t, bears).len(), mine.len());
    }
}

#[test]
fn twenty_goaded_creatures_under_crawlspace_all_attack() {
    cr!("508.1c", "508.1d", "701.15b", "802.3a");
    interp!("requirement-search-budget");
    for different in [false, true] {
        // Three players: P1 goaded P0's twenty creatures; P2 controls Crawlspace ("No more
        // than two creatures can attack you each combat"). All twenty attack (each obeys
        // "attacks each combat if able"), two of them P2 ("a player other than P1"),
        // eighteen P1.
        let mut t = TestGame::with_config(
            3,
            GameConfig {
                attack_multiple_players: true,
                ..Default::default()
            },
        );
        t.battlefield(P2, "Crawlspace");
        let mine = creatures(&mut t, P0, 20, different);
        for c in &mine {
            t.g.objects[c.0 as usize].goaded_by = vec![P1];
        }
        t.set_step(P0, Step::BeginningOfCombat);
        declare(&mut t, &[]);
        go_to(&mut t, Step::DeclareAttackers);
        let at = |p: PlayerId| {
            mine.iter()
                .filter(|c| attacks(&t, **c) == Some(Entity::Player(p)))
                .count()
        };
        assert_eq!((at(P1), at(P2)), (18, 2), "different: {different}");
    }
}

#[test]
fn twenty_goaded_mogg_flunkies_under_crawlspace_all_attack() {
    cr!("508.1c", "508.1d", "701.15b", "802.3a");
    interp!("requirement-search-budget");
    supported("Mogg Flunkies");
    // As above, with twenty Mogg Flunkies ("can't attack or block alone"): no creature can
    // be added to a declaration on its own, so only a search over how the alike creatures
    // split among P1, P2 and not attacking finds that all of them attack.
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    t.battlefield(P2, "Crawlspace");
    let mine: Vec<ObjectId> = (0..20)
        .map(|_| t.battlefield(P0, "Mogg Flunkies"))
        .collect();
    for c in &mine {
        t.g.objects[c.0 as usize].goaded_by = vec![P1];
    }
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    let at = |p: PlayerId| {
        mine.iter()
            .filter(|c| attacks(&t, **c) == Some(Entity::Player(p)))
            .count()
    };
    assert_eq!((at(P1), at(P2)), (18, 2));
}

#[test]
fn seventeen_creatures_that_must_block_all_block_with_an_attacker_only_one_can_block() {
    cr!("509.1b", "509.1c");
    interp!("requirement-search-budget");
    supported("Tahngarth, First Mate");
    for different in [false, true] {
        let mut t = TestGame::new(2);
        // Grand Melee; Tahngarth "can't be blocked by more than one creature".
        t.battlefield(P0, "Grand Melee");
        let tahngarth = t.battlefield(P0, "Tahngarth, First Mate");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let mine = creatures(&mut t, P1, 17, different);
        t.set_step(P0, Step::BeginningOfCombat);
        declare(
            &mut t,
            &[(tahngarth, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        // Not blocking with any of them is illegal: all seventeen can block.
        block(&mut t, P1, &[]);
        go_to(&mut t, Step::DeclareBlockers);
        assert!(
            blockers_of(&t, tahngarth).len() <= 1,
            "different: {different}"
        );
        assert_eq!(
            blockers_of(&t, tahngarth).len() + blockers_of(&t, bears).len(),
            mine.len(),
            "different: {different}"
        );
    }
}

#[test]
fn thirty_goaded_creatures_in_a_two_player_game_all_attack_their_goader() {
    cr!("508.1d", "701.15b");
    for different in [false, true] {
        // "Attacks a player other than [P1] if able" can't be obeyed: P1 is the only
        // opponent. Each creature still attacks (CR 701.15b).
        let mut t = TestGame::new(2);
        let mine = creatures(&mut t, P0, 30, different);
        for c in &mine {
            t.g.objects[c.0 as usize].goaded_by = vec![P1];
        }
        t.set_step(P0, Step::BeginningOfCombat);
        declare(
            &mut t,
            &mine[..29]
                .iter()
                .map(|c| (*c, Entity::Player(P1)))
                .collect::<Vec<_>>(),
        );
        go_to(&mut t, Step::DeclareAttackers);
        assert_eq!(attackers(&t).len(), 30, "different: {different}");
    }
}

#[test]
fn creatures_that_must_block_any_of_three_lure_creatures_each_block_one() {
    cr!("509.1c");
    supported("Prized Unicorn");
    // Prized Unicorn: "All creatures able to block this creature do so." Each Grizzly Bears
    // can block only one of them: it obeys one of its three requirements.
    let mut t = TestGame::new(2);
    let unicorns: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Prized Unicorn"))
        .collect();
    let mine = creatures(&mut t, P1, 10, false);
    t.set_step(P0, Step::BeginningOfCombat);
    let d: Vec<(ObjectId, Entity)> = unicorns.iter().map(|u| (*u, Entity::Player(P1))).collect();
    declare(&mut t, &d);
    block(&mut t, P1, &[(mine[0], unicorns[0])]);
    go_to(&mut t, Step::DeclareBlockers);
    for b in &mine {
        assert_eq!(blocking(&t, *b).len(), 1);
    }
}

#[test]
fn a_creature_that_can_block_two_lure_creatures_blocks_both() {
    cr!("509.1c");
    supported("Night Market Guard");
    // Night Market Guard "can block an additional creature each combat": blocking both
    // Prized Unicorns obeys both requirements, so blocking one is illegal.
    let mut t = TestGame::new(2);
    let u1 = t.battlefield(P0, "Prized Unicorn");
    let u2 = t.battlefield(P0, "Prized Unicorn");
    let guard = t.battlefield(P1, "Night Market Guard");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(u1, Entity::Player(P1)), (u2, Entity::Player(P1))],
    );
    block(&mut t, P1, &[(guard, u1)]);
    go_to(&mut t, Step::DeclareBlockers);
    let mut b = blocking(&t, guard);
    b.sort();
    let mut want = vec![u1, u2];
    want.sort();
    assert_eq!(b, want);
}

#[test]
fn trample_over_planeswalkers_reaches_the_defending_player_past_a_blocker_once_the_planeswalker_left(
) {
    cr!("702.19e", "702.19b");
    supported("Thrasta, Tempest's Roar");
    // Thrasta (7/7 trample, trample over planeswalkers) attacks Jace and is blocked by
    // Grizzly Bears; Jace leaves before combat damage. Lethal damage to the Bears, the rest
    // to Jace's controller (whom Thrasta isn't attacking).
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(thrasta, Entity::Object(jace))]);
    block(&mut t, P1, &[(bears, thrasta)]);
    go_to(&mut t, Step::DeclareBlockers);
    let owner = t.obj_now(jace).owner;
    t.g.move_object(
        jace,
        Zone::Graveyard(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
    assert!(t.g.is_attacking(thrasta));
    assert_eq!(attacks(&t, thrasta), None);
    t.script.lock().unwrap().asked.clear();
    // All 7 to the player: illegal (the Bears need lethal damage first), and replaced by
    // the default.
    assign(&mut t, P0, &[0, 7]);
    go_to(&mut t, Step::CombatDamage);
    let d = damage_decisions(&t);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].2, vec![Entity::Object(bears), Entity::Player(P1)]);
    assert_eq!(d[0].3, vec![2]);
    assert_eq!(t.life(P1), 15);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn an_attack_whose_cost_cant_be_paid_the_way_chosen_is_undone_and_its_attackers_untap() {
    cr!("508.1", "508.1f", "508.1j", "733.1", "107.4f");
    supported("Norn's Annex");
    // Norn's Annex: attacking P1 needs {W/P}. P0 has no white mana but 20 life, so the
    // attack can be paid for; P0 chooses to pay {W} rather than 2 life, and can't. The
    // declaration is illegal and the game returns to the moment before it: the Bears,
    // tapped as they were declared, are untapped again, and no creature attacks.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Norn's Annex");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    go_to(&mut t, Step::DeclareAttackers);
    assert!(
        t.asked()
            .iter()
            .any(|(p, d)| *p == P0
                && matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
    );
    assert!(attackers(&t).is_empty());
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.life(P0), 20);
}
