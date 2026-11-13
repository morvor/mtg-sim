//! Rulings batch S21 — "you choose which creatures block this combat and how those
//! creatures block" (Odric, Master Tactician; Brutal Hordechief): the attacking player
//! declares the defending player's blockers, following the rules for blocking (CR 509.1a–c).

use crate::r_s01_common::*;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn at_p1(attackers: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    attackers.iter().map(|a| (*a, Entity::Player(P1))).collect()
}

/// Who was asked to declare blockers since decision `from`.
fn block_deciders(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::DeclareBlockers { .. }))
        .map(|(p, _)| *p)
        .collect()
}

/// P0 attacks P1 with Odric and three other creatures (Odric's ability resolves), then
/// declares P1's blockers as `blocks`; the game stops after the declaration.
struct OdricCombat {
    t: TestGame,
    odric: ObjectId,
    giant: ObjectId,
    angel: ObjectId,
    bears: ObjectId,
    ogre: ObjectId,
    elves: ObjectId,
    wall: ObjectId,
}

fn odric_combat() -> OdricCombat {
    supported("Odric, Master Tactician");
    let mut t = TestGame::new(2);
    // "Whenever Odric and at least three other creatures attack, you choose which
    // creatures block this combat and how those creatures block."
    let odric = t.battlefield(P0, "Odric, Master Tactician");
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let wall = t.battlefield(P1, "Wall of Wood");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[odric, giant, angel, bears]));
    assert_eq!(triggers_on_stack(&t, "you choose which creatures block"), 1);
    t.resolve_all();
    OdricCombat {
        t,
        odric,
        giant,
        angel,
        bears,
        ogre,
        elves,
        wall,
    }
}

#[test]
fn odric_s_controller_declares_blocks_and_may_leave_creatures_out() {
    cr!("509.1", "509.1a");
    ruling!("Odric, Master Tactician", "You can decide that a creature won't block.");
    let OdricCombat {
        mut t,
        odric,
        giant,
        bears,
        ogre,
        elves,
        wall,
        ..
    } = odric_combat();
    // P0 has the Elves block the Hill Giant and the Ogre block Odric (first strike, 3/4);
    // the Wall of Wood doesn't block.
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(elves, giant), (ogre, odric)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(block_deciders(&t, from), vec![P0], "P0 declares, P1 doesn't");
    let mut b = blocks_now(&t);
    b.sort();
    let mut want = vec![(elves, giant), (ogre, odric)];
    want.sort();
    assert_eq!(b, want);
    assert!(!t.g.is_blocking(wall));
    t.advance_to(P0, Step::EndOfCombat);
    // The Serra Angel (4) and the Bears (2) were unblocked.
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(elves) && !t.on_battlefield(ogre));
    assert!(t.on_battlefield(bears));
}

#[test]
fn odric_s_controller_must_still_declare_legal_blocks() {
    cr!("509.1", "509.1a", "509.1b");
    ruling!("Odric, Master Tactician", "All blocking declarations must still be legal.");
    let OdricCombat {
        mut t,
        angel,
        bears,
        ogre,
        ..
    } = odric_combat();
    assert!(!legal_blocks(&mut t, P1, &[(ogre, angel)]));
    // P0 tries to have the Gray Ogre block the flying Serra Angel: not a legal
    // declaration. It's undone (and no creature blocks, the legal default).
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre, angel), (ogre, bears)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blocks_now(&t).is_empty());
}

#[test]
fn the_blocking_creature_s_controller_may_decline_to_pay_the_cost_of_odric_s_blocks() {
    cr!("509.1", "509.1d");
    ruling!(
        "Odric, Master Tactician",
        "If there's a cost associated with having a creature block and you choose for that creature to block, its controller can choose to pay that cost or not. If that player decides to not pay that cost, you must propose a new set of blocking creatures."
    );
    supported("Archangel of Tithes");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let odric = t.battlefield(P0, "Odric, Master Tactician");
        // "As long as this creature is attacking, creatures can't block unless their
        // controller pays {1} for each of those creatures."
        let archangel = t.battlefield(P0, "Archangel of Tithes");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.lands(P1, "Wastes", 1);
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &at_p1(&[odric, archangel, giant, bears]));
        t.resolve_all();
        // P0 has the Ogre block the Bears; P1 decides whether to pay {1}. If P1 doesn't,
        // P0 proposes again: no blocks.
        t.answer(
            P0,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(ogre, bears)]),
        );
        t.answer_yes(P1, pays);
        t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
        let from = t.asked().len();
        go_to(&mut t, Step::DeclareBlockers);
        let asked = &t.asked()[from..];
        assert!(asked
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
        let proposals = block_deciders(&t, from);
        if pays {
            assert_eq!(proposals, vec![P0]);
            assert_eq!(blocks_now(&t), vec![(ogre, bears)]);
            assert_eq!(tapped_lands(&t, P1), 1);
        } else {
            assert_eq!(proposals, vec![P0, P0], "P0 proposed a new set of blocks");
            assert!(blocks_now(&t).is_empty());
            assert_eq!(tapped_lands(&t, P1), 0);
        }
    }
}

#[test]
fn odric_needs_three_other_attackers() {
    cr!("509.1", "603.2");
    supported("Odric, Master Tactician");
    let mut t = TestGame::new(2);
    let odric = t.battlefield(P0, "Odric, Master Tactician");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[odric, giant, bears]));
    assert_eq!(triggers_on_stack(&t, "you choose which creatures block"), 0);
    let from = t.asked().len();
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre, bears)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(block_deciders(&t, from), vec![P1]);
    assert_eq!(blocks_now(&t), vec![(ogre, bears)]);
}

#[test]
fn brutal_hordechief_s_controller_chooses_blocks_that_obey_the_requirement() {
    cr!("509.1a", "509.1b", "509.1c");
    ruling!("Brutal Hordechief", "All blocking declarations must still be legal.");
    supported("Brutal Hordechief");
    let mut t = TestGame::new(2);
    // "{3}{R/W}{R/W}: Creatures your opponents control block this turn if able, and you
    // choose how those creatures block."
    let chief = t.battlefield(P0, "Brutal Hordechief");
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P0, "Serra Angel");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 5);
    activate_containing_text(&mut t, chief, "you choose how those creatures block");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant, angel]));
    t.resolve_all();
    // Each of P1's creatures must block if able: leaving the Elves out is illegal, and so
    // is having the Ogre block the flying Angel.
    assert!(!legal_blocks(&mut t, P1, &[(ogre, giant)]));
    assert!(!legal_blocks(&mut t, P1, &[(ogre, angel), (elves, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(ogre, giant), (elves, giant)]));
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre, giant), (elves, giant)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(block_deciders(&t, from), vec![P0]);
    let mut b = blocks_now(&t);
    b.sort();
    let mut want = vec![(ogre, giant), (elves, giant)];
    want.sort();
    assert_eq!(b, want);
}

#[test]
fn brutal_hordechief_applies_to_creatures_that_arrive_later() {
    cr!("509.1c", "611.2c");
    ruling!(
        "Brutal Hordechief",
        "You’ll choose how each creature controlled by an opponent blocks, even if that creature wasn’t on the battlefield or wasn’t controlled by an opponent as the activated ability resolved."
    );
    let mut t = TestGame::new(2);
    let chief = t.battlefield(P0, "Brutal Hordechief");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    activate_containing_text(&mut t, chief, "you choose how those creatures block");
    t.resolve_all();
    // After the ability resolved: a new creature for P1, and one P0 gave P1.
    let ogre = t.battlefield(P1, "Gray Ogre");
    let elves = t.battlefield(P0, "Llanowar Elves");
    crate::r_s06_common::give_control(&mut t, elves, P1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant]));
    t.resolve_all();
    // Both must block, and P0 declares their blocks.
    assert!(!legal_blocks(&mut t, P1, &[(ogre, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(ogre, giant), (elves, giant)]));
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre, giant), (elves, giant)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(block_deciders(&t, from), vec![P0]);
    assert_eq!(blocks_now(&t).len(), 2);
}

#[test]
fn with_two_hordechief_abilities_the_last_one_s_controller_chooses_for_the_others() {
    cr!("509.1", "802.4");
    ruling!(
        "Brutal Hordechief",
        "In a multiplayer game, if more than one player activates Brutal Hordechief’s activated ability on the same turn, the controller of the last ability to resolve will choose how any creatures controlled by players who didn’t resolve this ability will block."
    );
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chief1 = t.battlefield(P1, "Brutal Hordechief");
    let chief2 = t.battlefield(P2, "Brutal Hordechief");
    let ogre1 = t.battlefield(P1, "Gray Ogre");
    let ogre2 = t.battlefield(P2, "Gray Ogre");
    t.lands(P1, "Mountain", 5);
    t.lands(P2, "Mountain", 5);
    // P1's ability resolves first, then P2's.
    crate::r_s06_common::activate_containing(&mut t, P1, chief1, "you choose how")
        .expect("activate");
    t.resolve_all();
    crate::r_s06_common::activate_containing(&mut t, P2, chief2, "you choose how")
        .expect("activate");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    t.resolve_all();
    let from = t.asked().len();
    // P2 (last) declares P1's blocks; P1 declares P2's (P2's own ability doesn't apply to
    // P2's creatures). Every creature of theirs must block (the Hordechiefs too).
    t.answer(
        P2,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre1, giant), (chief1, giant)]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre2, bears), (chief2, bears)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    let mut deciders = block_deciders(&t, from);
    deciders.sort();
    assert_eq!(deciders, vec![P1, P2]);
    let asked = &t.asked()[from..];
    let decider_for = |blocker: ObjectId| {
        asked.iter().find_map(|(p, d)| match d {
            Decision::DeclareBlockers { options } if options.iter().any(|(b, _)| *b == blocker) => {
                Some(*p)
            }
            _ => None,
        })
    };
    assert_eq!(decider_for(ogre1), Some(P2));
    assert_eq!(decider_for(ogre2), Some(P1));
    let mut b = blocks_now(&t);
    b.sort();
    let mut want = vec![
        (ogre1, giant),
        (chief1, giant),
        (ogre2, bears),
        (chief2, bears),
    ];
    want.sort();
    assert_eq!(b, want);
}

/// Activates the activated ability of `source` (controlled by P0) whose text contains
/// `needle`.
fn activate_containing_text(t: &mut TestGame, source: ObjectId, needle: &str) {
    crate::r_s06_common::activate_containing(t, P0, source, needle).expect("activate");
}
