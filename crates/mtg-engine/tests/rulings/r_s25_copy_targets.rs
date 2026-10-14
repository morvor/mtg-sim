//! Rulings batch S25 — the targets of a copy of a spell (CR 707.10, 707.10c, 115.7d): the
//! copy has the original's targets unless its controller chooses new ones; any number of
//! them may be changed; a new target must be legal, and a target with no legal
//! alternative stays unchanged, even if it's illegal.

use crate::r_s01_common::supported;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s06_common::activate_containing;
use crate::r_s13_common::add;
use crate::r_s25_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 controls Grizzly Bears and Hill Giant. P0 casts Lightning Bolt at the Bears, then
/// `start` puts on the stack what will copy it (a spell or an ability; nothing if a
/// triggered ability already did), which resolves: P0 chooses the Hill Giant as the
/// copy's new target. Returns (bears, giant, bolt).
fn bolt_then_copy_at_the_giant(
    t: &mut TestGame,
    start: impl FnOnce(&mut TestGame, ObjectId),
) -> (ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = cast_new(t, P0, "Lightning Bolt", &[Entity::Object(bears)]);
    start(t, bolt);
    change_copy_targets(t, P0, &[Some(Entity::Object(giant))]);
    t.resolve();
    (bears, giant, bolt)
}

#[test]
fn a_copy_keeps_the_targets_unless_new_ones_are_chosen() {
    cr!("707.10", "707.10c", "115.7d");
    ruling!(
        "Twincast",
        "The copy will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Twincast");
    // New targets: the copy of Lightning Bolt kills the Hill Giant, the original the Bears.
    let mut t = TestGame::new(2);
    let (bears, giant, bolt) = bolt_then_copy_at_the_giant(&mut t, |t, bolt| {
        cast_new(t, P0, "Twincast", &[Entity::Object(bolt)]);
    });
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(giant)]);
    assert_eq!(targets_of(&t, bolt), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));

    // Any number of the targets may be changed: of Arc Trail's two targets, only the
    // first ("2 damage to any target") changes; the second stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let trail = cast_new(
        &mut t,
        P0,
        "Arc Trail",
        &[Entity::Object(bears), Entity::Player(P1)],
    );
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(trail)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant)), None]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(
        targets_of(&t, copy),
        vec![Entity::Object(giant), Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(giant).damage, 2);

    // A target with no legal alternative stays, although it's illegal now: the only
    // nonblack creature left the battlefield before Doom Blade was copied.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = cast_new(&mut t, P0, "Doom Blade", &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(blade)]);
    t.answer_yes(P0, true);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

#[test]
fn an_ability_copy_of_a_spell_may_get_new_targets() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Nivix Guildmage",
        "The copy will have the same targets as the spell it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can’t choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Nivix Guildmage");
    // "{2}{U}{R}: Copy target instant or sorcery spell you control. You may choose new
    // targets for the copy."
    let mut t = TestGame::new(2);
    let guildmage = t.battlefield(P0, "Nivix Guildmage");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    let (bears, _giant, bolt) = bolt_then_copy_at_the_giant(&mut t, |t, bolt| {
        t.answer_targets(P0, &[Entity::Object(bolt)]);
        activate_containing(t, P0, guildmage, "Copy target").unwrap();
    });
    assert_eq!(targets_of(&t, bolt), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Choosing no new targets keeps them.
    let mut t = TestGame::new(2);
    let guildmage = t.battlefield(P0, "Nivix Guildmage");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    activate_containing(&mut t, P0, guildmage, "Copy target").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(bears)]);
}

/// P1 controls Grizzly Bears, Hill Giant and Slippery Bogle (hexproof). P0 casts Lightning
/// Bolt at the Bears with `setup` having arranged a triggered copy of it; the copy's new
/// target must be legal: the Bogle isn't offered, the Hill Giant is.
fn new_targets_must_be_legal(setup: impl FnOnce(&mut TestGame)) {
    let mut t = TestGame::new(2);
    setup(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(bears)]);
    let from = t.asked().len();
    // P0 tries to choose the Bogle: it isn't a legal choice, so it isn't taken.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bogle)]);
    t.resolve();
    let offered = target_candidates(&t, P0, from);
    let last = offered.last().expect("no new target asked");
    assert!(last.contains(&Entity::Object(giant)));
    assert!(!last.contains(&Entity::Object(bogle)));
    let copy = spell_copies(&t)[0];
    assert_ne!(targets_of(&t, copy), vec![Entity::Object(bogle)]);
    assert_eq!(targets_of(&t, bolt), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bogle));
}

#[test]
fn a_delayed_copy_s_new_targets_must_be_legal() {
    cr!("707.10c", "115.7d", "702.11b");
    ruling!(
        "Teach by Example",
        "If the spell has any targets, the copy will have the same targets unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    supported("Teach by Example");
    // "When you next cast an instant or sorcery spell this turn, copy that spell. You may
    // choose new targets for the copy."
    new_targets_must_be_legal(|t| {
        cast_new(t, P0, "Teach by Example", &[]);
        t.resolve_all();
    });
}

#[test]
fn a_cast_trigger_copy_s_new_targets_must_be_legal() {
    cr!("707.10c", "115.7d", "702.11b");
    ruling!(
        "Double Vision",
        "The copy will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    supported("Double Vision");
    // "Whenever you cast your first instant or sorcery spell each turn, copy that spell.
    // You may choose new targets for the copy."
    new_targets_must_be_legal(|t| {
        t.battlefield(P0, "Double Vision");
    });
}

#[test]
fn an_emblem_copy_may_get_new_targets() {
    cr!("707.10c", "115.7d", "114.4");
    ruling!(
        "Will Kenrith",
        "The copy will have the same targets as the spell or ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Will Kenrith");
    // −8: Target player gets an emblem with "Whenever you cast an instant or sorcery
    // spell, copy it. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    let will = t.battlefield(P0, "Will Kenrith");
    add(&mut t, will, counters::LOYALTY, 4);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    activate_containing(&mut t, P0, will, "emblem").unwrap();
    t.resolve_all();
    let (bears, giant, bolt) = bolt_then_copy_at_the_giant(&mut t, |_, _| {});
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(giant)]);
    assert_eq!(targets_of(&t, bolt), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn a_target_with_no_legal_alternative_stays_unchanged() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Dual Strike",
        "If you copy a spell with targets, the copy will have the same targets unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for any of the targets, you can't choose a new legal target, that target remains unchanged (even if the current target is illegal)."
    );
    supported("Dual Strike");
    // "When you next cast an instant or sorcery spell with mana value 4 or less this turn,
    // copy that spell. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Dual Strike", &[]);
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = cast_new(&mut t, P0, "Doom Blade", &[Entity::Object(bears)]);
    // The Bears leave before the copy is made: there's no other nonblack creature.
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(bears)]);
    assert_eq!(targets_of(&t, blade), vec![Entity::Object(bears)]);
    // With a Hill Giant around, it could have been changed.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Dual Strike", &[]);
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Doom Blade", &[Entity::Object(bears)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant))]);
    t.resolve();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}
