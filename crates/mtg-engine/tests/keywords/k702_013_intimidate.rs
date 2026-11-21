//! CR 702.13 Intimidate.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn intimidate_is_an_evasion_ability() {
    cr!("702.13", "702.13a");
    assert!(KeywordKind::Intimidate.is_evasion());
    assert_supported("Bladetusk Boar");
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Bladetusk Boar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Evasion restricts blocks, not attacks: an unblockable-by-green creature still attacks
    // and deals damage if unblocked.
    attack_with(&mut t, &[(boar, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, boar));
    block_and_finish(&mut t, P1, &[(bears, boar)]);
    assert!(!is_blocking(&t, bears));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn intimidate_blocked_only_by_artifact_creatures_or_creatures_sharing_a_color() {
    cr!("702.13b");
    ruling!(
        "Bladetusk Boar",
        "Normally, Bladetusk Boar can't be blocked except by artifact creatures and/or red creatures."
    );
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Bladetusk Boar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let thopter = t.battlefield(P1, "Ornithopter");
    let swiftblade = t.battlefield(P1, "Boros Swiftblade");
    attack_with(&mut t, &[(boar, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, boar));
    assert!(t.g.can_block(goblin, boar));
    assert!(t.g.can_block(thopter, boar));
    assert!(t.g.can_block(swiftblade, boar));
    // An illegal block is undone (CR 509.1c); a legal one stands.
    block_and_finish(&mut t, P1, &[(bears, boar)]);
    assert!(!is_blocking(&t, bears));
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Bladetusk Boar");
    let thopter = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(boar, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(thopter, boar)]);
    assert_eq!(t.life(P1), 20);
}

/// Distorting Lens: "{T}: Target permanent becomes the color of your choice until end of
/// turn." — makes `id` blue.
fn make_blue(t: &mut TestGame, id: ObjectId) {
    let lens = t.battlefield(P0, "Distorting Lens");
    let i = Color::ALL.iter().position(|c| *c == Color::Blue).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.activate(P0, lens, 0, &[Entity::Object(id)]).unwrap();
    t.resolve();
}

#[test]
fn intimidate_uses_the_attackers_current_colors() {
    cr!("702.13b");
    ruling!(
        "Bladetusk Boar",
        "Intimidate looks at the current colors of a creature that has it."
    );
    ruling!(
        "Bladetusk Boar",
        "what colors it is matters only as the defending player declares blockers"
    );
    assert_supported("Distorting Lens");
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Bladetusk Boar");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let merfolk = t.battlefield(P1, "Coral Merfolk");
    make_blue(&mut t, boar);
    // Now blue: blue creatures may block it, red ones may not.
    attack_with(&mut t, &[(boar, Entity::Player(P1))]);
    assert!(!t.g.can_block(goblin, boar));
    assert!(t.g.can_block(merfolk, boar));
    block_and_finish(&mut t, P1, &[(merfolk, boar)]);
    assert!(t.in_graveyard(P1, "Coral Merfolk"));
    assert_eq!(t.life(P1), 20);
    // Once blocked, changing its colors doesn't undo the block.
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Bladetusk Boar");
    let goblin = t.battlefield(P1, "Raging Goblin");
    attack_with(&mut t, &[(boar, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(goblin, boar)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    make_blue(&mut t, boar);
    assert!(t.obj_now(boar).chars.colors.contains(Color::Blue));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(t.g.current(boar)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn multicolored_and_colorless_creatures_with_intimidate() {
    cr!("702.13b");
    ruling!(
        "Hideous Visage",
        "A multicolored creature with intimidate can be blocked by any creature that shares a color with it"
    );
    assert_supported("Hideous Visage");
    assert_supported("Executioner's Hood");
    let mut t = TestGame::new(2);
    let swiftblade = t.battlefield(P0, "Boros Swiftblade");
    let myr = t.battlefield(P0, "Darksteel Myr");
    let hood = t.battlefield(P0, "Executioner's Hood");
    t.g.attach(hood, Entity::Object(myr));
    t.lands(P0, "Swamp", 3);
    let visage = t.hand(P0, "Hideous Visage");
    t.cast(P0, visage).go();
    t.resolve_all();
    assert!(t.obj_now(swiftblade).has_keyword(KeywordKind::Intimidate));
    assert!(t.obj_now(myr).has_keyword(KeywordKind::Intimidate));
    let goblin = t.battlefield(P1, "Raging Goblin");
    let knight = t.battlefield(P1, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    attack_with(
        &mut t,
        &[(swiftblade, Entity::Player(P1)), (myr, Entity::Player(P1))],
    );
    // Red-white: red and white creatures may block it, green ones may not.
    assert!(t.g.can_block(goblin, swiftblade));
    assert!(t.g.can_block(knight, swiftblade));
    assert!(!t.g.can_block(bears, swiftblade));
    assert!(t.g.can_block(thopter, swiftblade));
    // Colorless: it shares a color with nothing, so only artifact creatures may block it.
    assert!(!t.g.can_block(goblin, myr));
    assert!(!t.g.can_block(knight, myr));
    assert!(t.g.can_block(thopter, myr));
}

#[test]
fn multiple_instances_of_intimidate_are_redundant() {
    cr!("702.13c");
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Bladetusk Boar");
    let hood = t.battlefield(P0, "Executioner's Hood");
    t.g.attach(hood, Entity::Object(boar));
    t.g.recompute();
    assert_eq!(keyword_count(&t, boar, KeywordKind::Intimidate), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let thopter = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(boar, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, boar));
    assert!(t.g.can_block(goblin, boar));
    assert!(t.g.can_block(thopter, boar));
}
