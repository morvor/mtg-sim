//! CR 704.5s / 704.5v audit: which triggered abilities on the stack keep a Saga or a
//! Siege on the battlefield. A Saga waits only for its chapter abilities (704.5s); a Siege
//! waits for any ability it's the source of (704.5v). A Saga with no chapter abilities
//! isn't sacrificed at all (714.2d, 714.4).

use crate::r603_audit_common::*;
use crate::r703_common::oracle_card;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// A two-chapter Saga that also has a triggered ability that isn't a chapter ability.
fn saga_with_another_trigger() -> mtg_engine::card::CardDef {
    oracle_card(
        "Lifewatch Saga",
        "Enchantment — Saga",
        "{0}",
        None,
        "I — You gain 1 life.\nII — You gain 2 life.\nWhenever you gain life, put a +1/+1 counter on target creature.",
    )
}

#[test]
fn a_saga_waits_only_for_its_chapter_abilities() {
    cr!("704.5s", "714.4", "714.1a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let saga = t.custom(P0, saga_with_another_trigger(), Zone::Battlefield);
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // Chapter II: the Saga stays while it's on the stack.
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(saga));
    // Chapter II resolves; the life gain triggers the Saga's other ability. That isn't a
    // chapter ability, so the Saga is sacrificed while it waits on the stack.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the life gain trigger");
    assert!(
        t.in_graveyard(P0, "Lifewatch Saga"),
        "not waiting for a non-chapter ability"
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn a_saga_creature_that_lost_its_chapter_abilities_isnt_sacrificed() {
    cr!("704.5s", "714.2d", "714.3c");
    supported("Humility");
    let mut t = TestGame::new(2);
    let saga = t.custom(
        P0,
        oracle_card(
            "Summoned Saga",
            "Enchantment Creature — Saga Spirit",
            "{0}",
            Some((3, 3)),
            "I — You gain 1 life.\nII — You gain 2 life.",
        ),
        Zone::Battlefield,
    );
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    t.battlefield(P1, "Humility");
    t.settle();
    // No chapter abilities: its final chapter number is 0, and the state-based action
    // applies only to a Saga with one or more chapter abilities.
    assert!(t.on_battlefield(saga));
    assert_eq!(t.pt(saga), (1, 1));
}

#[test]
fn a_saga_that_cant_be_sacrificed_stays_and_the_game_goes_on() {
    cr!("704.5s", "701.21a", "704.3");
    ruling!(
        "Assault Suit",
        "The equipped creature can't be sacrificed for any reason."
    );
    supported("Assault Suit");
    let mut t = TestGame::new(2);
    let saga = t.custom(
        P0,
        oracle_card(
            "Summoned Saga",
            "Enchantment Creature — Saga Spirit",
            "{0}",
            Some((3, 3)),
            "I — You gain 1 life.\nII — You gain 2 life.",
        ),
        Zone::Battlefield,
    );
    let suit = t.battlefield(P0, "Assault Suit");
    assert!(t.g.attach(suit, Entity::Object(saga)));
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    t.g.dirty = true;
    // Its controller can't sacrifice it: nothing happens, so the check doesn't repeat.
    assert!(!t.g.check_sbas());
    t.settle();
    assert!(t.on_battlefield(saga));
    assert!(t.g.result.is_none());
    // Once the Suit is gone, the Saga is sacrificed.
    t.g.destroy(suit, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Summoned Saga"));
}

#[test]
fn a_copy_of_a_chapter_ability_keeps_its_saga_until_it_leaves_the_stack() {
    cr!("704.5s", "707.10", "707.10b");
    interp!("copied-chapter-ability-keeps-saga");
    supported("Strionic Resonator");
    supported("Stifle");
    let mut t = TestGame::new(2);
    let saga = t.custom(
        P0,
        oracle_card(
            "Short Saga",
            "Enchantment — Saga",
            "{0}",
            None,
            "I — You gain 1 life.\nII — You gain 2 life.",
        ),
        Zone::Battlefield,
    );
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.resolve_all();
    let resonator = t.battlefield(P0, "Strionic Resonator");
    t.lands(P0, "Wastes", 2);
    // Chapter II triggers; Strionic Resonator copies it.
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.settle();
    let chapter = *t.g.stack.last().unwrap();
    t.activate(P0, resonator, 0, &[Entity::Object(chapter)])
        .expect("Strionic Resonator");
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, chapter);
    // The chapter ability that triggered is countered; its copy, with the Saga as its
    // source, is still on the stack.
    t.lands(P1, "Island", 2);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(chapter).go();
    t.resolve();
    assert_eq!(t.g.stack, vec![copy]);
    assert!(t.on_battlefield(saga), "the copy keeps the Saga");
    let life = t.life(P0);
    t.resolve();
    assert_eq!(t.life(P0), life + 2);
    assert!(t.in_graveyard(P0, "Short Saga"));
}
