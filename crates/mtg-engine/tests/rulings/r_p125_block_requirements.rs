//! Rulings batch P125 — "[creature] blocks [creature] this turn if able" (Lurking Arynx,
//! Feral Contest, Hunt Down, Rimehorn Aurochs, Avalanche Tusker): a requirement on blocks
//! that does nothing if the block is impossible or has a cost (CR 509.1c), and doesn't
//! make anyone attack.

use crate::r_p125_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

/// P0's Lurking Arynx and Craw Wurm (total power 9, so the Formidable ability can be
/// activated) at the beginning of combat; P1's Llanowar Elves and Gray Ogre.
fn arynx_board(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId, ObjectId) {
    supported("Lurking Arynx");
    let arynx = t.battlefield(P0, "Lurking Arynx");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let ogre = t.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(t, P0);
    (arynx, wurm, elves, ogre)
}

/// P0 activates Lurking Arynx targeting `target` and it resolves.
fn lure(t: &mut TestGame, arynx: ObjectId, target: ObjectId) {
    t.lands(P0, "Forest", 3);
    t.activate(P0, arynx, 0, &[obj(target)]).expect("activate");
    t.resolve_all();
}

#[test]
fn lurking_arynx_doesnt_have_to_attack() {
    cr!("509.1c", "508.1d");
    ruling!(
        "Lurking Arynx",
        "Activating Lurking Arynx’s ability doesn’t force you to attack with it that turn."
    );
    let mut t = TestGame::new(2);
    let (arynx, wurm, elves, _) = arynx_board(&mut t);
    lure(&mut t, arynx, elves);
    // Attacking with only the Wurm is legal; the Elves may block it or not block at all.
    attack_with(&mut t, &at_p1(&[wurm]));
    assert!(!t.g.combat.as_ref().unwrap().attacker(arynx).is_some());
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, wurm)]));
}

#[test]
fn lurking_arynx_requirement_does_nothing_if_the_block_is_impossible() {
    cr!("509.1c");
    ruling!(
        "Lurking Arynx",
        "If a creature affected by Lurking Arynx’s ability can’t legally block it (perhaps because Lurking Arynx has gained flying), that creature can block other attacking creatures or not block at all."
    );
    supported("Jump");
    let mut t = TestGame::new(2);
    let (arynx, wurm, elves, _) = arynx_board(&mut t);
    lure(&mut t, arynx, elves);
    let jump = t.hand(P0, "Jump");
    t.lands(P0, "Island", 1);
    t.cast(P0, jump).target(arynx).go();
    t.resolve_all();
    attack_with(&mut t, &at_p1(&[arynx, wurm]));
    assert!(!legal_blocks(&mut t, P1, &[(elves, arynx)]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, wurm)]));
}

#[test]
fn lurking_arynx_tapped_cant_block_and_costly_blockers_are_exempt() {
    cr!("509.1c");
    ruling!(
        "Lurking Arynx",
        "If, during the declare blockers step, a creature is tapped or is affected by a spell or ability that says it can’t block, it doesn’t block. If there’s a cost associated with having a creature block, its controller isn’t forced to pay that cost, so it doesn’t have to block in that case either."
    );
    // Without anything stopping it, the requirement applies.
    let mut t = TestGame::new(2);
    let (arynx, wurm, elves, _) = arynx_board(&mut t);
    lure(&mut t, arynx, elves);
    attack_with(&mut t, &at_p1(&[arynx, wurm]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, arynx)]));

    // Tapped: it doesn't block.
    let mut t = TestGame::new(2);
    let (arynx, wurm, elves, _) = arynx_board(&mut t);
    lure(&mut t, arynx, elves);
    t.g.tap(elves);
    attack_with(&mut t, &at_p1(&[arynx, wurm]));
    assert!(legal_blocks(&mut t, P1, &[]));
    go_to(&mut t, mtg_engine::turn::Step::DeclareBlockers);
    assert!(blocks_now(&t).is_empty());

    // A creature that can't block (Hulking Goblin: "This creature can't block.").
    supported("Hulking Goblin");
    let mut t = TestGame::new(2);
    let (arynx, wurm, _, _) = arynx_board(&mut t);
    let goblin = t.battlefield(P1, "Hulking Goblin");
    lure(&mut t, arynx, goblin);
    attack_with(&mut t, &at_p1(&[arynx, wurm]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(goblin, arynx)]));

    // A cost to block (Archangel of Tithes attacking: "creatures can't block unless their
    // controller pays {1} for each of those creatures"): P1 doesn't have to pay, so the
    // Elves don't block.
    supported("Archangel of Tithes");
    let mut t = TestGame::new(2);
    let (arynx, wurm, elves, _) = arynx_board(&mut t);
    let archangel = t.battlefield(P0, "Archangel of Tithes");
    lure(&mut t, arynx, elves);
    t.lands(P1, "Wastes", 1);
    attack_with(&mut t, &at_p1(&[arynx, wurm, archangel]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    go_to(&mut t, mtg_engine::turn::Step::DeclareBlockers);
    assert!(blocks_now(&t).is_empty());
}

#[test]
fn lurking_arynx_can_force_several_creatures_to_block_it() {
    cr!("509.1c");
    ruling!(
        "Lurking Arynx",
        "You may activate Lurking Arynx’s ability multiple times in a turn to force multiple creatures to block it if able."
    );
    let mut t = TestGame::new(2);
    let (arynx, wurm, elves, ogre) = arynx_board(&mut t);
    lure(&mut t, arynx, elves);
    lure(&mut t, arynx, ogre);
    attack_with(&mut t, &at_p1(&[arynx, wurm]));
    assert!(legal_blocks(&mut t, P1, &[(elves, arynx), (ogre, arynx)]));
    assert!(!legal_blocks(&mut t, P1, &[(elves, arynx)]));
    assert!(!legal_blocks(&mut t, P1, &[(elves, arynx), (ogre, wurm)]));
}

// --- Feral Contest -------------------------------------------------------------------------

/// P0 casts Feral Contest on `mine` (+1/+1 counter) and `other` (blocks it).
fn feral_contest(t: &mut TestGame, mine: ObjectId, other: ObjectId) -> ObjectId {
    supported("Feral Contest");
    cast_new(t, P0, "Feral Contest", &[obj(mine), obj(other)])
}

#[test]
fn feral_contest_doesnt_force_an_attack() {
    cr!("509.1c", "508.1d");
    ruling!(
        "Feral Contest",
        "Casting Feral Contest doesn’t force you to attack with the first targeted creature that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    feral_contest(&mut t, bears, elves);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, giant)]));
}

#[test]
fn feral_contest_first_target_illegal() {
    cr!("608.2b", "509.1c");
    ruling!(
        "Feral Contest",
        "If just the first targeted creature is an illegal target by the time Feral Contest resolves, it won’t get a +1/+1 counter. However, the second targeted creature is still affected by the blocking restriction"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    feral_contest(&mut t, bears, ogre);
    // In response, P0 gives the Bears protection from green with Gods Willing: an illegal
    // target for the green Feral Contest (the red Ogre can still block it).
    supported("Gods Willing");
    t.lands(P0, "Plains", 1);
    let gods = t.hand(P0, "Gods Willing");
    choose_color(&mut t, P0, Color::Green);
    t.cast(P0, gods).target(bears).go();
    t.resolve(); // Gods Willing
    t.resolve_all(); // Feral Contest
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, giant]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(ogre, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(ogre, bears)]));
}

#[test]
fn feral_contest_second_target_illegal() {
    cr!("608.2b", "509.1c");
    ruling!(
        "Feral Contest",
        "If just the second targeted creature is an illegal target by the time Feral Contest resolves, the first targeted creature will get a +1/+1 counter, but the second targeted creature won’t have to block it that turn."
    );
    supported("Gods Willing");
    // P1 controls the Elves and gives them protection from green in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    feral_contest(&mut t, bears, elves);
    t.lands(P1, "Plains", 1);
    let gods = t.hand(P1, "Gods Willing");
    choose_color(&mut t, P1, Color::Green);
    t.cast(P1, gods).target(elves).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    // The Elves have protection from green, so they can't block the green Bears anyway;
    // with an Ogre P1 may also choose not to block.
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn feral_contest_requirement_does_nothing_if_the_block_is_impossible() {
    cr!("509.1c");
    ruling!(
        "Feral Contest",
        "If you attack with the first targeted creature but the second targeted creature isn’t able to block it (for example, because the first targeted creature has flying and the second one doesn’t), the requirement to block does nothing."
    );
    supported("Jump");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    feral_contest(&mut t, bears, elves);
    t.resolve_all();
    let jump = t.hand(P0, "Jump");
    t.lands(P0, "Island", 1);
    t.cast(P0, jump).target(bears).go();
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, giant]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, giant)]));
}

#[test]
fn feral_contest_exempt_creatures_can_be_targeted() {
    cr!("509.1c", "115.1");
    ruling!(
        "Feral Contest",
        "Tapped creatures, creatures that can’t block as the result of an effect, creatures with unpaid costs to block (such as those from War Cadence), and creatures that aren’t controlled by the defending player are exempt from effects that would require them to block. Such creatures can be targeted by Feral Contest, but the requirement to block does nothing."
    );
    // A tapped creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(elves);
    feral_contest(&mut t, bears, elves);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(legal_blocks(&mut t, P1, &[]));

    // A creature that can't block (Hulking Goblin: "This creature can't block.").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let piker = t.battlefield(P1, "Hulking Goblin");
    feral_contest(&mut t, bears, piker);
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(legal_blocks(&mut t, P1, &[]));

    // A creature the defending player doesn't control (P0's own Hill Giant), in a
    // three-player game where P0 attacks P1.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    feral_contest(&mut t, bears, giant);
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, bears)]));
}

#[test]
fn feral_contest_needs_two_targets() {
    cr!("601.2c", "115.3");
    ruling!(
        "Feral Contest",
        "You must choose two targets as you cast Feral Contest: a creature you control and any other creature. If you can’t (because there’s only one creature on the battlefield, perhaps), then you can’t cast the spell. Note that the second target may also be a creature you control."
    );
    supported("Feral Contest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Feral Contest");
    let contest = t.hand(P0, "Feral Contest");
    assert!(!castable(&mut t, P0, contest));
    // With a second creature of P0's own, it can be cast.
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(castable(&mut t, P0, contest));
    t.cast(P0, contest).target(bears).target(giant).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

// --- Hunt Down, Rimehorn Aurochs ------------------------------------------------------------

#[test]
fn hunt_down_does_nothing_if_the_first_creature_cant_block_the_second() {
    cr!("509.1c");
    ruling!(
        "Hunt Down",
        "If the first creature targeted by Hunt Down can’t block the second targeted creature (for example, because the second creature has flying and the first doesn’t, or because both creatures are controlled by the same player), the ability does nothing"
    );
    supported("Hunt Down");
    // It works: P1's Elves must block P0's Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Hunt Down", &[obj(elves), obj(bears)]);
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, giant]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(elves, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(elves, bears)]));

    // The second creature has flying: the Elves are free.
    let mut t = TestGame::new(2);
    let birds = t.battlefield(P0, "Birds of Paradise");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Hunt Down", &[obj(elves), obj(birds)]);
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[birds, giant]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, giant)]));

    // Both creatures are P1's: nothing.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let ogre = t.battlefield(P1, "Gray Ogre");
    cast_new(&mut t, P0, "Hunt Down", &[obj(elves), obj(ogre)]);
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(elves, giant)]));
}

#[test]
fn rimehorn_aurochs_does_nothing_if_the_first_creature_cant_block_the_second() {
    cr!("509.1c", "602.2");
    ruling!(
        "Rimehorn Aurochs",
        "If the first creature targeted by Rimehorn Aurochs can’t block the second targeted creature (for example, because the second creature has flying and the first doesn’t, or because both creatures are controlled by the same player), the ability does nothing"
    );
    supported("Rimehorn Aurochs");
    for flying in [false, true] {
        let mut t = TestGame::new(2);
        let aurochs = t.battlefield(P0, "Rimehorn Aurochs");
        let attacker = t.battlefield(P0, if flying { "Birds of Paradise" } else { "Grizzly Bears" });
        let elves = t.battlefield(P1, "Llanowar Elves");
        t.lands(P0, "Snow-Covered Forest", 3);
        to_beginning_of_combat(&mut t, P0);
        t.activate(P0, aurochs, 0, &[obj(elves), obj(attacker)])
            .expect("activate");
        t.resolve_all();
        attack_with(&mut t, &at_p1(&[attacker, aurochs]));
        assert_eq!(legal_blocks(&mut t, P1, &[]), flying);
        assert_eq!(legal_blocks(&mut t, P1, &[(elves, aurochs)]), flying);
        assert!(legal_blocks(&mut t, P1, &[(elves, attacker)]) != flying);
    }
}

#[test]
fn avalanche_tusker_tapped_or_costly_blocker_doesnt_block() {
    cr!("509.1c");
    ruling!(
        "Avalanche Tusker",
        "If, during the declare blockers step, the target creature is tapped or is affected by a spell or ability that says it can’t block, then it doesn’t block. If there’s a cost associated with having that creature block, its controller isn’t forced to pay that cost."
    );
    supported("Avalanche Tusker");
    supported("Archangel of Tithes");
    // Tapped after the trigger resolves.
    let mut t = TestGame::new(2);
    let tusker = t.battlefield(P0, "Avalanche Tusker");
    let elves = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    t.answer_targets(P0, &[obj(elves)]);
    attack_with(&mut t, &at_p1(&[tusker]));
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[]));
    t.g.tap(elves);
    assert!(legal_blocks(&mut t, P1, &[]));

    // A cost to block: P1 doesn't pay it, and the Elves don't block.
    let mut t = TestGame::new(2);
    let tusker = t.battlefield(P0, "Avalanche Tusker");
    let archangel = t.battlefield(P0, "Archangel of Tithes");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P1, "Wastes", 1);
    to_beginning_of_combat(&mut t, P0);
    t.answer_targets(P0, &[obj(elves)]);
    attack_with(&mut t, &at_p1(&[tusker, archangel]));
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    go_to(&mut t, mtg_engine::turn::Step::DeclareBlockers);
    assert!(blocks_now(&t).is_empty());
    assert!(t.g.combat.as_ref().unwrap().attacker(tusker).is_some());
}
