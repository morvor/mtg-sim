//! Rulings batch P158 — combat: first strike gained after the first combat damage step
//! (CR 510.4), pump spells affecting only the creatures controlled as they resolve
//! (CR 611.2c), a creature that blocked stays blocking (CR 506.4), a blocked creature stays
//! blocked (CR 509.1h), and sacrifices chosen by the affected player as the spell resolves
//! in APNAP order and performed at once (CR 608.2c, 101.4, 701.17).

use crate::r_p158_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// Runs the game until the first-strike combat damage has been dealt and the active
/// player holds priority in that step.
fn after_first_strike_damage(t: &mut TestGame) {
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::FirstStrikeDamage
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "no first-strike damage step");
    t.settle();
}

#[test]
fn first_strike_gained_after_first_strike_damage_still_deals_regular_damage() {
    cr!("510.4", "702.7b", "702.33a");
    ruling!(
        "Bold Defense",
        "If a creature doesn't have first strike, granting it first strike after combat damage has been dealt in the first combat damage step won't prevent it from dealing combat damage. It will still assign and deal its combat damage in the second combat damage step."
    );
    supported("Bold Defense");
    supported("White Knight");
    // P0's Hill Giant (3/3) attacks; P1's White Knight (2/2 first strike) blocks. After
    // the Knight's first-strike damage, kicked Bold Defense gives the Giant +2/+2 and
    // first strike: it still deals its 5 damage in the regular step.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let knight = t.battlefield(P1, "White Knight");
    attack_with(&mut t, &[(giant, pl(P1))]);
    crate::r_p148_common::declare_blocks(&mut t, P1, &[(knight, giant)]);
    after_first_strike_damage(&mut t);
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(knight).damage, 0);
    lots(&mut t, P0);
    let b = t.hand(P0, "Bold Defense");
    t.cast(P0, b).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    assert!(crate::r_s06_common::has_kw(&t, giant, mtg_engine::keywords::KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(knight), "the Giant dealt its damage");
    assert!(t.on_battlefield(giant));
}

#[test]
fn team_pumps_affect_only_the_creatures_controlled_as_they_resolve() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Bold Defense",
        "Only creatures you control as Bold Defense resolves are affected."
    );
    ruling!(
        "Pride of Conquerors",
        "Pride of Conquerors affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get a bonus."
    );
    supported("Pride of Conquerors");
    for spell in ["Bold Defense", "Pride of Conquerors"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        lots(&mut t, P0);
        let s = t.hand(P0, spell);
        t.cast(P0, s).kicked(false).go();
        // A creature that enters in response is affected.
        let ogre = t.battlefield(P0, "Gray Ogre");
        t.resolve_all();
        assert_eq!(t.pt(bears), (3, 3), "{spell}");
        assert_eq!(t.pt(ogre), (3, 3), "{spell}");
        // Not one that enters afterwards, nor one gained afterwards.
        let later = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(t.pt(later), (2, 2), "{spell}");
        cast_at(&mut t, P0, "Ray of Command", &[obj(theirs)]);
        t.resolve_all();
        assert_eq!(ctl(&t, theirs), P0);
        assert_eq!(t.pt(theirs), (3, 3), "{spell}");
    }
}

#[test]
fn a_creature_that_blocked_stays_blocking_after_the_attacker_dies() {
    cr!("506.4", "509.1h");
    ruling!(
        "Will of the All-Hunter",
        "Once a creature blocks, it remains a blocking creature through the end of combat step, even if the attacking creature dies or leaves combat."
    );
    supported("Will of the All-Hunter");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let blocker = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(attacker, pl(P0))]);
    crate::r_p148_common::declare_blocks(&mut t, P0, &[(blocker, attacker)]);
    kill(&mut t, attacker);
    assert!(!t.on_battlefield(attacker));
    cast_at(&mut t, P0, "Will of the All-Hunter", &[obj(blocker)]);
    t.resolve_all();
    assert_eq!(t.counters(blocker, "+1/+1"), 2, "it was blocking");
    assert_eq!(t.pt(blocker), (5, 5));
    // Not blocking: +2/+2 instead.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_at(&mut t, P0, "Will of the All-Hunter", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 0);
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn the_destined_thief_cant_unblock_a_blocked_creature() {
    cr!("509.1h", "506.4");
    ruling!(
        "The Destined Thief",
        "Once a creature you control has been blocked, activating The Destined Thief's second ability targeting that creature won't cause it to become unblocked."
    );
    supported("The Destined Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "The Destined Thief");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, pl(P1))]);
    crate::r_p148_common::declare_blocks(&mut t, P1, &[(giant, bears)]);
    act(&mut t, P0, thief, 0, &[obj(bears)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked: no damage to P1");
    assert_eq!(t.obj_now(giant).damage, 2);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn dispense_justice_lets_the_target_player_choose_as_it_resolves() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Dispense Justice",
        "The targeted player chooses which attacking creature(s) to sacrifice as Dispense Justice resolves."
    );
    supported("Dispense Justice");
    for (metalcraft, sacrificed) in [(false, 1), (true, 2)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        if metalcraft {
            for _ in 0..3 {
                t.battlefield(P0, "Ornithopter");
            }
        }
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P1, "Gray Ogre");
        attack_with(&mut t, &[(a, pl(P0)), (b, pl(P0)), (c, pl(P0))]);
        cast_at(&mut t, P0, "Dispense Justice", &[pl(P1)]);
        let from = t.asked().len();
        // P1 keeps the Giant.
        let keep: Vec<Entity> = if metalcraft { vec![obj(a), obj(c)] } else { vec![obj(a)] };
        t.answer_choose(P1, &keep);
        t.resolve();
        let chooser: Vec<PlayerId> = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(chooser, vec![P1], "metalcraft {metalcraft}");
        assert!(t.on_battlefield(b), "metalcraft {metalcraft}");
        assert_eq!(creatures(&t, P1).len(), 3 - sacrificed, "metalcraft {metalcraft}");
        assert!(!t.on_battlefield(a));
    }
}

/// P0 casts Extract a Confession, collecting evidence (exiling a Craw Wurm from the
/// graveyard) when `evidence`.
fn extract(t: &mut TestGame, evidence: bool) {
    supported("Extract a Confession");
    lots(t, P0);
    let s = t.hand(P0, "Extract a Confession");
    if evidence {
        let wurm = t.graveyard(P0, "Craw Wurm");
        t.answer_choose(P0, &[obj(wurm)]);
    }
    t.cast(P0, s).kicked(evidence).go();
    assert_eq!(t.in_exile("Craw Wurm"), evidence);
}

#[test]
fn extract_a_confession_ties_are_broken_by_the_sacrificing_player() {
    cr!("608.2c", "701.17a", "701.59a", "701.59c");
    ruling!(
        "Extract a Confession",
        "If evidence was collected and an opponent has multiple creatures tied for the greatest power, that player chooses which one to sacrifice."
    );
    // Two 3/3s and a 2/2: P1 picks which 3/3 goes.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Grizzly Bears");
    extract(&mut t, true);
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(b)]);
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(a) && t.on_battlefield(c));
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(a)) && offered[0].contains(&obj(b)) && !offered[0].contains(&obj(c)), "{offered:?}");
}

#[test]
fn extract_a_confession_opponents_choose_in_turn_order_then_sacrifice_at_once() {
    cr!("101.4", "101.4a", "608.2c", "701.17a");
    ruling!(
        "Extract a Confession",
        "Starting with the next opponent in turn order (or, if you cast Extract a Confession on an opponent's turn, starting with the opponent whose turn it is) and proceeding in turn order, each opponent chooses a creature they control to sacrifice. Then those creatures are sacrificed at the same time."
    );
    supported("Vedalken Orrery");
    // On P0's turn: P1 chooses, then P2 (with P1's creature still on the battlefield).
    // On P2's turn (Vedalken Orrery lets P0 cast it then): P2 chooses, then P1.
    for active in [P0, P2] {
        let mut t = TestGame::new(3);
        if active == P2 {
            t.battlefield(P0, "Vedalken Orrery");
            t.set_step(P2, Step::PrecombatMain);
        }
        let p1s = t.battlefield(P1, "Grizzly Bears");
        let p2s = t.battlefield(P2, "Hill Giant");
        let seen = crate::r_s01_common::watch(
            &mut t,
            P2,
            |d| matches!(d, Decision::ChooseEntities { .. }),
            |g| g.battlefield.iter().filter(|o| g.obj(**o).is(CardType::Creature)).count(),
        );
        extract(&mut t, false);
        let from = t.asked().len();
        t.resolve_all();
        let order: Vec<PlayerId> = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
            .map(|(p, _)| *p)
            .collect();
        let expected = if active == P0 { vec![P1, P2] } else { vec![P2, P1] };
        assert_eq!(order, expected, "active {active:?}");
        assert!(!t.on_battlefield(p1s) && !t.on_battlefield(p2s), "active {active:?}");
        // Both creatures were still there when P2 chose: nothing is sacrificed until
        // everyone has chosen.
        assert_eq!(*seen.lock().unwrap(), vec![2], "active {active:?}");
    }
}
