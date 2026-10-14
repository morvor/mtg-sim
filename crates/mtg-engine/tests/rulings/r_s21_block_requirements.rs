//! Rulings batch S21 — requirements on blocks (CR 509.1c): "all creatures able to block
//! this creature do so", "must be blocked if able", and what they don't require (creatures
//! that can't block, costs to block).

use crate::r_s01_common::*;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn p1() -> Entity {
    Entity::Player(P1)
}

fn at_p1(attackers: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    attackers.iter().map(|a| (*a, p1())).collect()
}

/// P1 declares `blocks` in the declare blockers step, and the game stops after it with
/// the blocks made (an illegal declaration is replaced by a legal one, CR 509.1).
fn declare_blocks(t: &mut TestGame, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    go_to(t, Step::DeclareBlockers);
}

#[test]
fn a_creature_required_to_block_two_attackers_blocks_the_one_its_controller_chooses() {
    cr!("509.1a", "509.1c");
    ruling!(
        "Nessian Boar",
        "If a creature is required to block two or more different creatures, its controller chooses which one that creature blocks."
    );
    supported("Nessian Boar");
    supported("Treeshaker Chimera");
    // Each: "All creatures able to block this creature do so."
    for chosen in 0..2 {
        let mut t = TestGame::new(2);
        let boar = t.battlefield(P0, "Nessian Boar");
        let chimera = t.battlefield(P0, "Treeshaker Chimera");
        let bears = t.battlefield(P1, "Grizzly Bears");
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &at_p1(&[boar, chimera]));
        // Blocking either one is legal; not blocking isn't.
        assert!(legal_blocks(&mut t, P1, &[(bears, boar)]));
        assert!(legal_blocks(&mut t, P1, &[(bears, chimera)]));
        assert!(!legal_blocks(&mut t, P1, &[]));
        let target = [boar, chimera][chosen];
        declare_blocks(&mut t, &[(bears, target)]);
        assert_eq!(blocks_now(&t), vec![(bears, target)]);
    }
}

#[test]
fn only_one_creature_must_block_the_creature_that_must_be_blocked() {
    cr!("509.1c");
    ruling!(
        "Compelled Duel",
        "Only one creature is required to block the affected creature. Other creatures may also block it and are free to block other creatures or not block at all."
    );
    supported("Compelled Duel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Gray Ogre");
    let z = t.battlefield(P1, "Wall of Wood");
    // "Target creature gets +3/+3 until end of turn and must be blocked this turn if able."
    let duel = t.hand(P0, "Compelled Duel");
    t.lands(P0, "Forest", 2);
    t.cast(P0, duel).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, giant]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears), (y, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears), (y, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(z, bears), (x, giant), (y, giant)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(x, giant), (y, giant)]));
    declare_blocks(&mut t, &[(z, bears), (y, giant)]);
    let mut b = blocks_now(&t);
    b.sort();
    let mut want = vec![(z, bears), (y, giant)];
    want.sort();
    assert_eq!(b, want);
}

#[test]
fn a_creature_that_must_be_blocked_isnt_if_no_creature_can_block_it_for_free() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Enlarge",
        "If each creature the defending player controls can't block for any reason (such as being tapped), then the affected creature isn't blocked. If there's a cost associated with blocking the affected creature, the defending player isn't forced to pay that cost, so it doesn't have to be blocked in that case either."
    );
    supported("Enlarge");
    supported("Archangel of Tithes");
    // "Target creature gets +7/+7 and gains trample until end of turn. It must be blocked
    // this turn if able."
    let enlarged = |t: &mut TestGame| {
        let bears = t.battlefield(P0, "Grizzly Bears");
        let enlarge = t.hand(P0, "Enlarge");
        t.lands(P0, "Forest", 5);
        t.cast(P0, enlarge).target(bears).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), (9, 9));
        bears
    };
    // P1's only creature is tapped: the Bears aren't blocked.
    let mut t = TestGame::new(2);
    let bears = enlarged(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, &[]);
    assert!(t.g.combat.as_ref().unwrap().is_unblocked(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 11);

    // Blocking costs {1} (Archangel of Tithes is attacking): P1 needn't pay it.
    let mut t = TestGame::new(2);
    let bears = enlarged(&mut t);
    let angel = t.battlefield(P0, "Archangel of Tithes");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, angel]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(giant, bears)]));
    declare_blocks(&mut t, &[]);
    assert!(t.g.combat.as_ref().unwrap().is_unblocked(bears));
    assert_eq!(tapped_lands(&t, P1), 0);
}

#[test]
fn lure_doesnt_make_tapped_or_restricted_creatures_block_nor_force_a_cost() {
    cr!("509.1a", "509.1b", "509.1c", "509.1d");
    ruling!(
        "Lure",
        "As blockers are declared, any creature that’s tapped or affected by a spell or ability that says it can’t block doesn’t block. If there’s a cost associated with having the creature block, no player is forced to pay that cost, so it doesn’t block if that cost isn’t paid."
    );
    supported("Lure");
    supported("Pacifism");
    // "All creatures able to block enchanted creature do so."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Lure", bears);
    let tapped = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(tapped);
    let pacified = t.battlefield(P1, "Gray Ogre");
    attach_new(&mut t, P0, "Pacifism", pacified);
    let free = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    // Only the Hill Giant is able to block: it must; the others don't.
    assert!(legal_blocks(&mut t, P1, &[(free, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(free, bears), (tapped, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(free, bears), (pacified, bears)]));
    declare_blocks(&mut t, &[]);
    // The illegal "no blocks" declaration was replaced: the Hill Giant blocks.
    assert_eq!(blocks_now(&t), vec![(free, bears)]);

    // With a cost to block (Archangel of Tithes attacking), nothing has to block.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Lure", bears);
    let angel = t.battlefield(P0, "Archangel of Tithes");
    let free = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, angel]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(free, bears)]));
    declare_blocks(&mut t, &[]);
    assert!(blocks_now(&t).is_empty());
}

#[test]
fn each_creature_that_must_be_blocked_gets_a_blocker_if_one_could_block_it() {
    cr!("509.1c");
    ruling!(
        "Satyr Piper",
        "During the declare blockers step, the defending player must assign at least one blocker to each creature that must be blocked if that player controls any creatures that could block it."
    );
    supported("Satyr Piper");
    let mut t = TestGame::new(2);
    // "{3}{G}: Target creature must be blocked this turn if able."
    let piper = t.battlefield(P0, "Satyr Piper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let flyer = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Forest", 12);
    for target in [bears, giant, flyer] {
        t.activate(P0, piper, 0, &[Entity::Object(target)])
            .expect("activate");
        t.resolve_all();
    }
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, giant, flyer]));
    // Nothing P1 controls could block the Serra Angel (flying); each of the others must
    // get a blocker.
    assert!(legal_blocks(&mut t, P1, &[(x, bears), (y, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(x, giant), (y, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, bears), (y, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, &[(x, bears), (y, bears)]);
    let blocked: Vec<ObjectId> = blocks_now(&t).into_iter().map(|(_, a)| a).collect();
    assert!(blocked.contains(&bears) && blocked.contains(&giant));
    assert!(t.g.combat.as_ref().unwrap().is_unblocked(flyer));
}
