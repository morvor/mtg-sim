//! Rulings batch P113 — targets chosen by an opponent (CR 601.7, 603.3d): who controls
//! the ability, hexproof, which opponent chooses, and a chooser who left the game
//! (CR 800.4g).

use crate::r_p113_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The players asked to choose targets since `from`, with the candidates offered.
fn target_choices(t: &TestGame, from: usize) -> Vec<(PlayerId, Vec<Entity>)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn cuombajj_witches_opponent_chooses_but_you_control_the_ability() {
    cr!("601.7", "702.11b", "115.1c");
    ruling!(
        "Cuombajj Witches",
        "Even though the opponent chooses a target, you control the ability. An opponent can't target a creature they control with hexproof."
    );
    supported("Cuombajj Witches");
    let mut t = TestGame::new(2);
    let witches = t.battlefield(P0, "Cuombajj Witches");
    let mine = t.battlefield(P0, "Gladecover Scout");
    let theirs = t.battlefield(P1, "Gladecover Scout");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P1, &[ent(mine)]);
    t.activate(P0, witches, 0, &[]).expect("activate");
    let asked = target_choices(&t, from);
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0].0, P0, "you choose first");
    assert_eq!(asked[1].0, P1);
    // P1 can't target their own hexproof creature (the ability is P0's), but can target
    // P0's.
    assert!(!asked[1].1.contains(&ent(theirs)));
    assert!(asked[1].1.contains(&ent(mine)));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn cuombajj_witches_you_choose_which_opponent_chooses() {
    cr!("601.7a", "601.7");
    ruling!(
        "Cuombajj Witches",
        "You choose which opponent chooses the second target. It doesn't have to be the opponent who controls (or is) the first target."
    );
    let mut t = TestGame::new(3);
    let witches = t.battlefield(P0, "Cuombajj Witches");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_targets(P2, &[ent(bears)]);
    t.activate(P0, witches, 0, &[]).expect("activate");
    let choosers: Vec<_> = target_choices(&t, from).into_iter().map(|x| x.0).collect();
    assert_eq!(choosers, vec![P0, P2]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(crate::r_s07_common::damage_on(&t, bears), 1);
}

/// P0's Bladegriff Prototype attacks P1 unblocked in a four-player game.
fn bladegriff(p1_life: i32) -> (TestGame, usize, ObjectId, ObjectId) {
    supported("Bladegriff Prototype");
    let mut t = TestGame::new(4);
    let griff = t.battlefield(P0, "Bladegriff Prototype");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P2, "Hill Giant");
    t.g.players[P1.idx()].life = p1_life;
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(griff, Entity::Player(P1))]);
    let from = t.asked().len();
    t.answer_targets(P1, &[ent(other)]);
    // If P1 left the game, P0 picks P3 to choose in their place.
    t.answer_choose(P0, &[Entity::Player(P3)]);
    t.answer_targets(P3, &[ent(other)]);
    block_and_finish(&mut t, P1, &[]);
    (t, from, theirs, other)
}

#[test]
fn bladegriff_prototype_damaged_player_chooses_but_you_control_it() {
    cr!("603.3d", "601.2c", "510.2");
    ruling!(
        "Bladegriff Prototype",
        "Even though the player who was dealt damage chooses the target for Bladegriff Prototype's ability, you are the controller of the ability."
    );
    let (t, from, theirs, other) = bladegriff(20);
    let asked = target_choices(&t, from);
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, P1);
    // P1 chooses among permanents P0's opponents control: P1's own and P2's.
    assert!(asked[0].1.contains(&ent(theirs)) && asked[0].1.contains(&ent(other)));
    assert!(!t.on_battlefield(other), "P1 chose P2's Hill Giant");
    assert!(t.on_battlefield(theirs));
}

#[test]
fn bladegriff_prototype_another_opponent_chooses_if_that_player_left() {
    cr!("800.4g", "603.3d");
    ruling!(
        "Bladegriff Prototype",
        "If an opponent dealt damage by Bladegriff Prototype leaves the game before the triggered ability goes on the stack"
    );
    let (t, from, theirs, other) = bladegriff(2);
    assert!(t.has_lost(P1));
    // P0 chooses which of the remaining opponents chooses the target.
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseEntities { candidates, .. }
            if candidates == &vec![Entity::Player(P2), Entity::Player(P3)])));
    let asked = target_choices(&t, from);
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, P3);
    assert!(!asked[0].1.contains(&ent(theirs)), "P1's permanents left with P1");
    assert!(!t.on_battlefield(other), "P3 chose P2's Hill Giant");
}
