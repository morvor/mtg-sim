//! Referent grammar III, batches (CR 603.2c, 608.2c, 400.7): "one of them" of a "one or
//! more" trigger, and "them" for the cards a batch discarded.

use mtg_engine::kwa::suspect_detain::is_suspected;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn toluz_exiles_the_discarded_cards_from_your_graveyard() {
    cr!("603.2c", "400.7");
    compiles("Toluz, Clever Conductor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toluz, Clever Conductor");
    let a = t.hand(P0, "Lightning Bolt");
    let b = t.hand(P0, "Island");
    let kept = t.graveyard(P0, "Grizzly Bears");
    t.g.discard(P0, a, None);
    t.g.discard(P0, b, None);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Island"));
    // A card that was already in the graveyard stays.
    assert_eq!(t.zone(kept), Zone::Graveyard(P0));
}

#[test]
fn conspiracy_theorist_exiles_one_of_the_discarded_cards_and_lets_you_cast_it() {
    cr!("603.2c");
    ruling!(
        "Conspiracy Theorist",
        "The card exiled by the second ability is exiled face up. Casting it follows the normal rules for casting that card."
    );
    compiles("Conspiracy Theorist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conspiracy Theorist");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    t.g.discard(P0, bolt, None);
    t.g.discard(P0, shock, None);
    let bolt_gy = t.g.current(bolt);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt_gy)]);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Shock"));
    let exiled = t.g.current(bolt);
    t.cast(P0, exiled).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn descendants_fury_sacrifices_one_of_the_creatures_that_dealt_damage() {
    cr!("603.2c", "701.21a");
    ruling!(
        "Descendants' Fury",
        "You'll reveal cards until you find one that shares at least one creature type with the creature you sacrificed."
    );
    compiles("Descendants' Fury");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Descendants' Fury");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.library_top(P0, "Runeclaw Bear");
    t.library_top(P0, "Island");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.attack(
        &[(bear, Entity::Player(P1)), (elf, Entity::Player(P1))],
        &[],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(bear), "the chosen creature was sacrificed");
    assert!(t.on_battlefield(elf));
    // Runeclaw Bear shares the Bear type with the sacrificed creature.
    assert_eq!(t.named_on_battlefield("Runeclaw Bear").len(), 1);
}

#[test]
fn frantic_scapegoat_passes_suspicion_to_one_of_the_creatures_that_entered() {
    cr!("603.2c", "701.60a");
    compiles("Frantic Scapegoat");
    let mut t = TestGame::new(2);
    let goat = t.enter(P0, "Frantic Scapegoat");
    t.resolve_all();
    assert!(is_suspected(&t.g, goat));
    let bear = t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.resolve_all();
    assert!(is_suspected(&t.g, bear));
    assert!(!is_suspected(&t.g, goat));
}
