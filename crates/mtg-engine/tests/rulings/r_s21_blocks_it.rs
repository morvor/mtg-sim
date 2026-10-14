//! Rulings batch S21 — "[creature] blocks it this turn if able", where "it" is a creature
//! the text named earlier (a target, the source, or the trigger object): a requirement on
//! blocks (CR 509.1c), including one to block a creature with menace (CR 702.111b).

use crate::r_s01_common::*;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn at_p1(attackers: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    attackers.iter().map(|a| (*a, Entity::Player(P1))).collect()
}

/// P0 casts Monstrous Step ("Target creature gets +7/+7 until end of turn. Up to one other
/// target creature blocks it this turn if able.") on `pumped`, making `blocker` block it.
fn monstrous_step(t: &mut TestGame, pumped: ObjectId, blocker: ObjectId) {
    let step = t.hand(P0, "Monstrous Step");
    t.lands(P0, "Forest", 5);
    t.cast(P0, step).target(pumped).target(blocker).go();
    t.resolve_all();
}

#[test]
fn a_creature_required_to_block_a_menace_creature_needs_another_blocker_with_it() {
    cr!("509.1b", "509.1c", "702.111b");
    ruling!(
        "Monstrous Step",
        "If a creature is required to block a creature with menace, another creature must also block that creature if able. If none can, the creature that’s required to block can block another creature or not block at all."
    );
    supported("Monstrous Step");
    supported("Boggart Brute");
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Boggart Brute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Gray Ogre");
    monstrous_step(&mut t, brute, x);
    assert_eq!(t.pt(brute), (10, 9));
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[brute, bears]));
    // The Elves must block the Brute, so the Ogre must block it too.
    assert!(legal_blocks(&mut t, P1, &[(x, brute), (y, brute)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, brute)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, bears), (y, bears)]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    go_to(&mut t, Step::DeclareBlockers);
    let mut b = blocks_now(&t);
    b.sort();
    let mut want = vec![(x, brute), (y, brute)];
    want.sort();
    assert_eq!(b, want, "the illegal declaration was replaced by the legal one");

    // If no other creature can block the Brute (the Ogre is tapped), the Elves may block
    // another creature or not block at all.
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Boggart Brute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Gray Ogre");
    monstrous_step(&mut t, brute, x);
    t.g.tap(y);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[brute, bears]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, brute)]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(x, bears)]),
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(blocks_now(&t), vec![(x, bears)]);
}

#[test]
fn creatures_other_than_the_one_required_may_also_block_the_monstrous_step_creature() {
    cr!("509.1c");
    ruling!(
        "Monstrous Step",
        "Creatures other than the second target creature may still block the first target creature if able."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Gray Ogre");
    monstrous_step(&mut t, bears, x);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears), (y, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(y, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
}

#[test]
fn avalanche_tuskers_target_blocks_it_if_able() {
    cr!("509.1c", "603.2");
    ruling!(
        "Avalanche Tusker",
        "If the target creature can’t block Avalanche Tusker, but could block another attacking creature, it’s free to block that creature or block no creatures at all."
    );
    supported("Avalanche Tusker");
    supported("Jump");
    // "Whenever this creature attacks, target creature defending player controls blocks
    // it this combat if able."
    let mut t = TestGame::new(2);
    let tusker = t.battlefield(P0, "Avalanche Tusker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(x)]);
    attack_with(&mut t, &at_p1(&[tusker, bears]));
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[(x, tusker)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[]));

    // The target can't block the Tusker (it has flying: P0 cast Jump on it in the
    // beginning of combat step): the Elves are free to block the Bears or not block.
    let mut t = TestGame::new(2);
    let tusker = t.battlefield(P0, "Avalanche Tusker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    let jump = t.hand(P0, "Jump");
    t.lands(P0, "Island", 1);
    t.cast(P0, jump).target(tusker).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(x)]);
    attack_with(&mut t, &at_p1(&[tusker, bears]));
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, tusker)]));
}

#[test]
fn impetuous_devils_target_doesnt_block_if_tapped() {
    cr!("509.1a", "509.1c");
    ruling!(
        "Impetuous Devils",
        "If a creature affected by the first triggered ability of Impetuous Devils is tapped or is affected by a spell or ability that says it can’t block, then it doesn’t block."
    );
    supported("Impetuous Devils");
    // "When this creature attacks, up to one target creature defending player controls
    // blocks it this combat if able."
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        let devils = t.battlefield(P0, "Impetuous Devils");
        let x = t.battlefield(P1, "Gray Ogre");
        to_beginning_of_combat(&mut t, P0);
        t.answer_targets(P0, &[Entity::Object(x)]);
        attack_with(&mut t, &at_p1(&[devils]));
        t.resolve_all();
        if tapped {
            t.g.tap(x);
        }
        assert_eq!(legal_blocks(&mut t, P1, &[]), tapped);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
        go_to(&mut t, Step::DeclareBlockers);
        let want = if tapped { vec![] } else { vec![(x, devils)] };
        assert_eq!(blocks_now(&t), want);
    }
}

#[test]
fn fighter_class_can_require_one_creature_to_block_two_attackers() {
    cr!("509.1c", "716.2a");
    ruling!(
        "Fighter Class",
        "If the last triggered ability triggers more than once in the same combat, a single creature may be required to block more than one creature."
    );
    supported("Fighter Class");
    use crate::r_s19_common::{gain_level, level};
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Fighter Class");
    t.lands(P0, "Plains", 5);
    t.lands(P0, "Mountain", 5);
    for n in [2, 3] {
        gain_level(&mut t, P0, class, n).expect("level up");
        t.resolve_all();
    }
    assert_eq!(level(&t, class), 3);
    // "Whenever a creature you control attacks, up to one target creature blocks it this
    // combat if able." Both triggers target the Ogre.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let elves = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    attack_with(&mut t, &at_p1(&[bears, giant]));
    t.resolve_all();
    // The Ogre blocks either attacker (its controller chooses); the Elves are free.
    assert!(legal_blocks(&mut t, P1, &[(ogre, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(ogre, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(ogre, giant), (elves, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(elves, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
}
