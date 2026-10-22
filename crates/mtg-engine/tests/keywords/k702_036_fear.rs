//! CR 702.36 Fear.

use crate::common_k702_011_017::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn fear_is_an_evasion_ability() {
    cr!("702.36", "702.36a");
    assert!(KeywordKind::Fear.is_evasion());
    assert_supported("Dross Prowler");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Dross Prowler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(prowler, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, prowler));
    block_and_finish(&mut t, P1, &[(bears, prowler)]);
    assert!(!is_blocking(&t, bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn fear_can_be_blocked_only_by_artifact_or_black_creatures() {
    cr!("702.36b");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Dross Prowler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    // Black, artifact, and both.
    let corpse = t.battlefield(P1, "Walking Corpse");
    let thopter = t.battlefield(P1, "Ornithopter");
    let myr = t.battlefield(P1, "Darksteel Myr");
    attack_with(&mut t, &[(prowler, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, prowler));
    assert!(!t.g.can_block(goblin, prowler));
    assert!(t.g.can_block(corpse, prowler));
    assert!(t.g.can_block(thopter, prowler));
    assert!(t.g.can_block(myr, prowler));
    block_and_finish(&mut t, P1, &[(corpse, prowler)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Walking Corpse"));
}

#[test]
fn a_creature_with_fear_can_block_normally() {
    cr!("702.36b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let prowler = t.battlefield(P1, "Dross Prowler");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.g.can_block(prowler, bears));
}

#[test]
fn multiple_instances_of_fear_are_redundant() {
    cr!("702.36c");
    assert_supported("Cover of Darkness");
    let mut t = TestGame::new(2);
    // Cover of Darkness naming Zombie gives Dross Prowler (a Zombie with fear) a second
    // instance of fear.
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Zombie")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let cover = t.hand(P0, "Cover of Darkness");
    t.lands(P0, "Swamp", 2);
    t.cast(P0, cover).go();
    t.resolve();
    t.clear_answers();
    let prowler = t.battlefield(P0, "Dross Prowler");
    assert_eq!(keyword_count(&t, prowler, KeywordKind::Fear), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let corpse = t.battlefield(P1, "Walking Corpse");
    attack_with(&mut t, &[(prowler, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, prowler));
    assert!(t.g.can_block(corpse, prowler));
}
