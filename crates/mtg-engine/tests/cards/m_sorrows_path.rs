//! Sorrow's Path (hand-written, `src/cards/sorrows_path.rs`): two blockers controlled by
//! the same opponent swap what they block, if each could block the other's attackers.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn blocking(t: &TestGame, b: ObjectId) -> Vec<ObjectId> {
    t.g.combat.as_ref().map(|c| c.blocking(b)).unwrap_or_default()
}

/// P0 attacks with both; P1 blocks `a1` with `b1` and `a2` with `b2`; stops in the declare
/// blockers step.
fn blocks(t: &mut TestGame, a1: ObjectId, b1: ObjectId, a2: ObjectId, b2: ObjectId) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a1, Entity::Player(P1)), (a2, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, a1), (b2, a2)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
}

#[test]
fn the_blockers_swap_what_they_block() {
    cr!("506.4", "509.1");
    ruling!("Sorrow's Path", "Whether those creatures could block all creatures the other is blocking isn’t determined until the ability resolves.");
    ruling!("Sorrow's Path", "The second ability triggers any time it becomes tapped, whether to pay for its ability or not.");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, wurm, wall, giant, bears);
    t.answer_targets(P0, &[Entity::Object(wall), Entity::Object(bears)]);
    t.activate(P0, path, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(blocking(&t, wall), vec![giant]);
    assert_eq!(blocking(&t, bears), vec![wurm]);
    // Its other ability triggered as it became tapped.
    assert_eq!(t.life(P0), 18);
}

#[test]
fn nothing_happens_if_one_couldnt_block_the_others_attacker() {
    cr!("509.1b");
    ruling!("Sorrow's Path", "take into account evasion abilities (like flying)");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let angel = t.battlefield(P0, "Serra Angel");
    let giant = t.battlefield(P0, "Hill Giant");
    let spider = t.battlefield(P1, "Giant Spider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, angel, spider, giant, bears);
    t.answer_targets(P0, &[Entity::Object(spider), Entity::Object(bears)]);
    t.activate(P0, path, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(blocking(&t, spider), vec![angel]);
    assert_eq!(blocking(&t, bears), vec![giant]);
}

#[test]
fn a_blocker_whose_attackers_left_combat_takes_over_and_the_other_blocks_nothing() {
    cr!("506.4", "509.1");
    ruling!("Sorrow's Path", "if all the creatures that one of the targeted creatures was blocking have left combat");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, wurm, wall, giant, bears);
    mtg_engine::combat::remove_from_combat(&mut t.g, giant);
    t.answer_targets(P0, &[Entity::Object(wall), Entity::Object(bears)]);
    t.activate(P0, path, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(blocking(&t, bears), vec![wurm]);
    assert!(!t.g.is_blocking(wall));
}

#[test]
fn the_two_blockers_must_be_controlled_by_the_same_opponent() {
    cr!("115.1");
    let mut t = TestGame::new(3);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let bears = t.battlefield(P2, "Grizzly Bears");
    let bears2 = t.battlefield(P2, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(wurm, Entity::Player(P1)), (giant, Entity::Player(P2))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(wall, wurm)]));
    t.answer(
        P2,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, giant), (bears2, giant)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.is_blocking(wall) && t.g.is_blocking(bears));
    // Blockers of two different opponents aren't a legal pair of targets (the answer is
    // refused and a legal pair is chosen instead). In this variant neither could block
    // the other's attacker anyway (CR 802.4a), so only the targets tell.
    t.answer_targets(P0, &[Entity::Object(wall), Entity::Object(bears)]);
    t.activate(P0, path, 0, &[]).unwrap();
    let ability = t
        .g
        .stack
        .iter()
        .copied()
        .find(|s| t.obj(*s).stack.as_ref().is_some_and(|x| !x.chosen.is_empty()))
        .expect("the ability on the stack");
    let targets = t.obj(ability).stack.as_ref().unwrap().chosen[0].targets[0].clone();
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&Entity::Object(bears)) || targets.contains(&Entity::Object(bears2)));
    assert!(!targets.contains(&Entity::Object(wall)));
}
