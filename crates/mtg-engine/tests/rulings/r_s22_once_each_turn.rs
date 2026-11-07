//! Rulings batch S22 — Karador, Ghost Chieftain ("Once during each of your turns, you may
//! cast a creature spell from your graveyard."): a permission to cast one creature spell
//! from the graveyard each of its controller's turns (CR 601.3), paying its costs or an
//! alternative cost instead (CR 118.9), following the timing rules (CR 307.1, 302.1).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn karador_the_spell_is_cast_paying_its_costs_or_an_alternative_cost() {
    cr!("601.3", "601.2f", "601.2h", "118.9", "702.74a");
    ruling!(
        "Karador, Ghost Chieftain",
        "You must pay the costs to cast that spell. If it has an alternative cost, you may cast it for that cost instead."
    );
    supported("Karador, Ghost Chieftain");
    supported("Mulldrifter");
    // Grizzly Bears ({1}{G}) needs its mana cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty(), "no mana");
    t.lands(P0, "Forest", 2);
    assert_eq!(legal_cast_methods(&mut t, P0, bears), vec![CastMethod::Normal]);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Mulldrifter may be cast for its evoke cost {2}{U} instead of {4}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let md = t.graveyard(P0, "Mulldrifter");
    add_mana(&mut t, P0, ManaType::U, 3);
    let methods = legal_cast_methods(&mut t, P0, md);
    assert_eq!(methods, vec![CastMethod::Keyword(KeywordKind::Evoke)]);
    let hand = t.hand_size(P0);
    t.cast(P0, md)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "Mulldrifter's enters trigger drew two");
    assert!(t.in_graveyard(P0, "Mulldrifter"), "evoked: sacrificed");
    // Cast for its evoke cost, it was still cast with Karador's permission: no other
    // creature spell from the graveyard this turn.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
}

#[test]
fn karador_once_during_each_of_your_turns() {
    cr!("601.3", "500.1");
    ruling!(
        "Karador, Ghost Chieftain",
        "You must follow the normal timing permissions and restrictions of the spell you cast from your graveyard."
    );
    ruling!(
        "Karador, Ghost Chieftain",
        "If you cast one creature spell from your graveyard and then have a new Karador come under your control in the same turn, you may cast another creature spell from your graveyard that turn."
    );
    supported("Karador, Ghost Chieftain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 6);
    // Not while a spell is on the stack (a creature spell's timing).
    let bolt = t.hand(P0, "Lightning Bolt");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
    t.resolve_all();
    t.cast(P0, bears).go();
    t.resolve_all();
    // Once: Llanowar Elves can't be cast from the graveyard this turn.
    assert!(legal_cast_methods(&mut t, P0, elves).is_empty());
    // A second Karador gives another permission this turn.
    t.battlefield(P0, "Karador, Ghost Chieftain");
    assert!(!legal_cast_methods(&mut t, P0, elves).is_empty());
    // Not during another player's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.set_step(P1, Step::PrecombatMain);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
    // On P0's next turn, again.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!legal_cast_methods(&mut t, P0, bears).is_empty());
}

#[test]
fn karador_a_card_cast_with_an_effects_permission_doesnt_use_it() {
    cr!("601.3");
    supported("Karador, Ghost Chieftain");
    supported("Emry, Lurker of the Loch");
    // Emry ("{T}: Choose target artifact card in your graveyard. You may cast that card
    // this turn.") lets P0 cast Ornithopter, an artifact creature card, from the
    // graveyard: that uses Emry's permission, and Karador's is still there for Grizzly
    // Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    let thopter = t.graveyard(P0, "Ornithopter");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, emry, 0, &[Entity::Object(thopter)])
        .expect("activate Emry");
    t.resolve_all();
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    t.lands(P0, "Forest", 2);
    assert_eq!(legal_cast_methods(&mut t, P0, bears), vec![CastMethod::Normal]);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
