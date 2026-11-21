//! CR 702.111 Menace.

use crate::common_k702_111_124::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn menace_is_an_evasion_ability() {
    cr!("702.111", "702.111a");
    assert!(KeywordKind::Menace.is_evasion());
    let mut t = TestGame::new(2);
    // Boggart Brute: 3/2 menace.
    let brute = t.battlefield(P0, "Boggart Brute");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare_attack(&mut t, &[(brute, Entity::Player(P1))]);
    // A lone creature can't block it: that declaration is illegal, and the engine
    // declares a legal one instead (no blocks, CR 509.1).
    assert!(!legal_blocks(&t, P1, &[(bears, brute)]));
    finish_combat(&mut t, P1, &[(bears, brute)]);
    assert!(!t.g.is_blocking(bears));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_creature_with_menace_can_be_blocked_by_two_or_more_creatures() {
    cr!("702.111b");
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Boggart Brute");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    declare_attack(&mut t, &[(brute, Entity::Player(P1))]);
    assert!(legal_blocks(&t, P1, &[]));
    assert!(!legal_blocks(&t, P1, &[(a, brute)]));
    assert!(legal_blocks(&t, P1, &[(a, brute), (b, brute)]));
    assert!(legal_blocks(&t, P1, &[(a, brute), (b, brute), (c, brute)]));
    finish_combat(&mut t, P1, &[(a, brute), (b, brute)]);
    assert!(t.in_graveyard(P0, "Boggart Brute"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn menace_only_restricts_blocking_it() {
    cr!("702.111b");
    let mut t = TestGame::new(2);
    // A creature with menace blocks normally, alone.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let brute = t.battlefield(P1, "Boggart Brute");
    declare_attack(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(legal_blocks(&t, P1, &[(brute, bears)]));
    finish_combat(&mut t, P1, &[(brute, bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn removing_a_blocker_doesnt_undo_the_block() {
    cr!("702.111b");
    ruling!(
        "Boggart Brute",
        "Once an attacking creature with menace is legally blocked by two or more creatures, removing one or more of those blockers from combat won't change or undo that block."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Boggart Brute");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    declare_attack(&mut t, &[(brute, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![(a, brute), (b, brute)]),
    );
    to_step(&mut t, Step::DeclareBlockers);
    mtg_engine::combat::remove_from_combat(&mut t.g, a);
    mtg_engine::combat::remove_from_combat(&mut t.g, b);
    to_step(&mut t, Step::EndOfCombat);
    // Still blocked: it deals no combat damage to the player.
    assert_eq!(t.life(P1), 20);
}

#[test]
fn gaining_menace_after_blockers_are_declared_doesnt_undo_a_block() {
    cr!("702.111b");
    ruling!(
        "Drey Keeper",
        "Gaining menace after blockers are declared won't undo a block that's already been done."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    declare_attack(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![(wall, bears)]),
    );
    to_step(&mut t, Step::DeclareBlockers);
    gain(&mut t, P0, bears, Keyword::new(KeywordKind::Menace));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    to_step(&mut t, Step::EndOfCombat);
    assert!(t.g.is_blocking(wall));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn menace_and_cant_be_blocked_by_more_than_one_creature_means_unblockable() {
    cr!("702.111b");
    ruling!(
        "Bristling Boar",
        "If Bristling Boar gains menace, it can’t be blocked at all."
    );
    let mut t = TestGame::new(2);
    // Bristling Boar: 4/3, "can't be blocked by more than one creature."
    let boar = t.battlefield(P0, "Bristling Boar");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    gain(&mut t, P0, boar, Keyword::new(KeywordKind::Menace));
    declare_attack(&mut t, &[(boar, Entity::Player(P1))]);
    assert!(!legal_blocks(&t, P1, &[(a, boar)]));
    assert!(!legal_blocks(&t, P1, &[(a, boar), (b, boar)]));
    finish_combat(&mut t, P1, &[(a, boar), (b, boar)]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn multiple_instances_of_menace_are_redundant() {
    cr!("702.111c");
    ruling!(
        "Bog Badger",
        "Multiple instances of menace on the same creature are redundant."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Boggart Brute");
    gain(&mut t, P0, brute, Keyword::new(KeywordKind::Menace));
    assert_eq!(t.obj_now(brute).chars.keyword_count(KeywordKind::Menace), 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    declare_attack(&mut t, &[(brute, Entity::Player(P1))]);
    // Still just "two or more", not four or more.
    assert!(!legal_blocks(&t, P1, &[(a, brute)]));
    assert!(legal_blocks(&t, P1, &[(a, brute), (b, brute)]));
}

#[test]
fn bog_badger_gives_menace_to_creatures_you_control_when_kicked() {
    cr!("702.111b");
    assert_supported_card("Bog Badger");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Swamp", 1);
    let badger = t.hand(P0, "Bog Badger");
    t.cast(P0, badger).kicked(true).go();
    t.resolve_all();
    assert!(has(&t, bears, KeywordKind::Menace));
    assert!(has(&t, badger, KeywordKind::Menace));
    let wall = t.battlefield(P1, "Wall of Stone");
    declare_attack(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&t, P1, &[(wall, bears)]));
}
