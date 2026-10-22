//! CR 508.1d, 509.1c: the declarations of attackers and of blockers must obey as many
//! requirements as possible, however many creatures there are. With seventeen creatures
//! that must attack (or block), too many declarations to try one by one, declaring none
//! is still illegal.

use crate::r506_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn attacking(t: &TestGame) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.attackers.iter().map(|a| a.id).collect())
        .unwrap_or_default()
}

#[test]
fn seventeen_creatures_that_attack_if_able_all_attack() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    // "All creatures attack each combat if able. All creatures block each combat if able."
    t.battlefield(P1, "Grand Melee");
    let bears = t.lands(P0, "Grizzly Bears", 17);
    // P0 declares no attackers, disobeying all seventeen requirements.
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    let attacking = attacking(&t);
    assert_eq!(attacking.len(), 17);
    assert!(bears.iter().all(|b| attacking.contains(b)));
}

#[test]
fn seventeen_creatures_that_block_if_able_all_block() {
    cr!("509.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grand Melee");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.lands(P1, "Grizzly Bears", 17);
    declare(&mut t, &[(giant, Entity::Player(P1))]);
    // P1 declares no blockers, disobeying all seventeen requirements.
    block(&mut t, P1, &[]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(attacking(&t), vec![giant]);
    let blocking: Vec<ObjectId> = t
        .g
        .combat
        .as_ref()
        .map(|c| c.blockers.iter().map(|b| b.id).collect())
        .unwrap_or_default();
    assert_eq!(blocking.len(), 17);
    assert!(bears.iter().all(|b| blocking.contains(b)));
}
