//! CR 702.9 Flying.

use super::k702_001_010_common::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn flying_is_an_evasion_ability_parsed_from_keyword_lines() {
    cr!("702.9a");
    // Serra Angel's "Flying" line compiles to the flying keyword, an evasion ability.
    let kws = printed_keywords("Serra Angel", KeywordKind::Flying);
    assert_eq!(kws.len(), 1);
    assert!(KeywordKind::Flying.is_evasion());
    assert_supported("Wind Drake");
}

#[test]
fn flyer_cant_be_blocked_except_by_flying_or_reach() {
    cr!("702.9b", "509.1b");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spider = t.battlefield(P1, "Giant Spider");
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(drake, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(bears, drake), "no flying or reach");
    assert!(t.g.can_block(spider, drake), "reach");
    assert!(t.g.can_block(angel, drake), "flying");

    // An illegal block by the Bears is undone; the drake connects.
    block(&mut t, P1, &[(bears, drake)]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert_eq!(damage(&t, bears), 0);
}

#[test]
fn flyer_can_block_creatures_with_or_without_flying() {
    cr!("702.9b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P0, "Wind Drake");
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (drake, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.can_block(angel, bears));
    assert!(t.g.can_block(angel, drake));
    block(&mut t, P1, &[(angel, bears)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn flying_granted_by_a_spell_works_like_printed_flying() {
    cr!("702.9b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    let jump = t.hand(P0, "Jump");
    let r = t.cast(P0, jump).target(bears).try_go();
    assert!(r.is_ok(), "{r:?}");
    t.resolve();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(giant, bears));
}

#[test]
fn multiple_instances_of_flying_are_redundant() {
    cr!("702.9c");
    let mut t = TestGame::new(2);
    // Wind Drake has flying, and Levitation gives it flying again.
    let drake = t.battlefield(P0, "Wind Drake");
    t.battlefield(P0, "Levitation");
    grant(&mut t, drake, Keyword::new(KeywordKind::Flying));
    assert_eq!(instances(&t, drake, KeywordKind::Flying), 3);
    let spider = t.battlefield(P1, "Giant Spider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(drake, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    // Same blocking rules as with one instance: one creature with reach can block it.
    assert!(t.g.can_block(spider, drake));
    assert!(!t.g.can_block(bears, drake));
    assert_eq!(t.g.min_blockers(drake), 1);
    // Losing flying removes every instance.
    remove_kw(&mut t, drake, KeywordKind::Flying);
    assert_eq!(instances(&t, drake, KeywordKind::Flying), 0);
    assert!(t.g.can_block(bears, drake));
}

#[test]
fn a_single_reach_blocker_stops_a_creature_with_two_instances_of_flying() {
    cr!("702.9c", "702.9b");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    t.battlefield(P0, "Levitation");
    assert_eq!(instances(&t, drake, KeywordKind::Flying), 2);
    let spider = t.battlefield(P1, "Giant Spider");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(drake, Entity::Player(P1))]);
    block(&mut t, P1, &[(spider, drake)]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage(&t, spider), 2);
    assert!(!t.on_battlefield(drake));
}

#[test]
fn reach_creature_blocks_a_flyer() {
    cr!("702.9b", "702.17b");
    ruling!(
        "Giant Spider",
        "a creature with flying can only be blocked by creatures with flying or reach"
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let spider = t.battlefield(P1, "Giant Spider");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(angel, Entity::Player(P1))]);
    block(&mut t, P1, &[(spider, angel)]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(spider));
}

#[test]
fn gaining_flying_after_being_blocked_doesnt_change_the_block() {
    cr!("702.9b", "509.1b");
    ruling!(
        "Jump",
        "Once a creature has become blocked, giving it flying won’t change that."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    let jump = t.hand(P0, "Jump");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, bears)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(is_blocked(&t, bears));
    t.cast(P0, jump).target(bears).go();
    t.resolve();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(is_blocked(&t, bears));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(bears));
}
