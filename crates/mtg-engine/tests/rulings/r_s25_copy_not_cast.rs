//! Rulings batch S25 — a copy of a spell is put onto the stack, not cast (CR 707.10), so
//! abilities that trigger when a player casts a spell (CR 603.2) don't trigger for it;
//! casting a copy of a card (CR 707.12) is casting a spell.

use crate::r_s01_common::{supported, with_subtype};
use crate::r_s19_common::add_lore;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Elemental tokens Young Pyromancer made for P0 ("Whenever you cast an instant or
/// sorcery spell, create a 1/1 red Elemental creature token.").
fn elementals(t: &TestGame) -> usize {
    with_subtype(t, P0, "Elemental").len()
}

#[test]
fn copying_a_spell_with_a_spell_doesnt_trigger_cast_abilities() {
    cr!("707.10", "603.2");
    ruling!(
        "Twincast",
        "The copy is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(bolt)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Lightning Bolt and Twincast were cast; the copy wasn't.
    assert_eq!(elementals(&t), 2);
    assert_eq!(t.g.history.spells_cast.len(), 2);
}

#[test]
fn the_mirari_conjecture_s_copies_dont_trigger_it_again() {
    cr!("707.10", "603.2", "714.2b");
    ruling!(
        "The Mirari Conjecture",
        "The copy is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger."
    );
    supported("The Mirari Conjecture");
    // III — "Until end of turn, whenever you cast an instant or sorcery spell, copy it.
    // You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let saga = t.battlefield(P0, "The Mirari Conjecture");
    add_lore(&mut t, saga, 3);
    t.resolve_all();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    // One copy: the copy doesn't trigger the chapter's ability (or Young Pyromancer).
    assert_eq!(t.life(P1), 14);
    assert_eq!(elementals(&t), 1);
}

#[test]
fn a_saga_s_delayed_copy_doesnt_trigger_cast_abilities() {
    cr!("707.10", "603.2", "603.7");
    ruling!(
        "Gadwick's First Duel",
        "The copy is created on the stack, so it's not \"cast.\" Creating the copy won't cause abilities that trigger when a player casts a spell to trigger."
    );
    supported("Gadwick's First Duel");
    // III — "When you next cast an instant or sorcery spell with mana value 3 or less this
    // turn, copy that spell. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let saga = t.battlefield(P0, "Gadwick's First Duel");
    add_lore(&mut t, saga, 3);
    t.resolve_all();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(elementals(&t), 1);
}

#[test]
fn a_casualty_copy_given_by_silverquill_isnt_cast() {
    cr!("707.10", "603.2", "702.153a");
    ruling!(
        "Silverquill, the Disputant",
        "The copy of the spell is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Silverquill, the Disputant");
    // "Each instant and sorcery spell you cast has casualty 1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill, the Disputant");
    t.battlefield(P0, "Young Pyromancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // The copy has casualty too, but wasn't cast: no second copy, no second Elemental.
    assert_eq!(elementals(&t), 1);
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn a_conspire_copy_doesnt_conspire_again() {
    cr!("707.10", "603.2", "702.78a");
    ruling!(
        "Raiding Schemes",
        "(such as conspire's triggered ability that creates a copy of the spell) won't trigger."
    );
    supported("Raiding Schemes");
    // "Each noncreature spell you cast has conspire."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raiding Schemes");
    let g1 = t.battlefield(P0, "Goblin Piker");
    let g2 = t.battlefield(P0, "Goblin Piker");
    let g3 = t.battlefield(P0, "Goblin Piker");
    let g4 = t.battlefield(P0, "Goblin Piker");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(g1), Entity::Object(g2)]);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert!(t.obj_now(g1).tapped && t.obj_now(g2).tapped);
    // Two more untapped red creatures are around, but the copy wasn't cast.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(g3), Entity::Object(g4)]);
    t.resolve();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(!t.obj_now(g3).tapped && !t.obj_now(g4).tapped);
}

#[test]
fn a_staff_triggers_for_a_cast_copy_of_a_card_but_not_a_copy_of_a_spell() {
    cr!("707.10", "707.12", "603.2");
    ruling!(
        "Staff of the Flame Magus",
        "If you cast a copy of a card that’s a certain color, the appropriate Staff’s ability will trigger. However, if you copy a spell on the stack without casting that copy, it will not."
    );
    supported("Staff of the Flame Magus");
    supported("Isochron Scepter");
    // "Whenever you cast a red spell or a Mountain you control enters, you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Staff of the Flame Magus");
    // Twincast (blue) copies Lightning Bolt: only the Bolt was a red spell cast.
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(bolt)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 21);
    // Isochron Scepter: "You may copy the exiled card. If you do, you may cast the copy
    // without paying its mana cost." Casting the copy of Lightning Bolt triggers it.
    let card = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, scepter, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
    assert_eq!(t.life(P0), 22);
}
