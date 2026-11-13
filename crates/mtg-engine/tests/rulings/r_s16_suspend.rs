//! Rulings on suspend (CR 702.62): its three abilities, and cards with no mana cost.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::next_upkeep;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

/// Suspends `card` for `p` (paying its suspend cost); returns the card in exile.
fn suspend(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.perform_action(p, Action::Special(SpecialAction::Suspend { card }))
        .expect("couldn't suspend");
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    exiled
}

#[test]
fn suspend_is_a_static_ability_and_two_triggered_abilities() {
    cr!("702.62a");
    ruling!(
        "Search for Tomorrow",
        "Suspend is a keyword that represents three abilities. The first is a static ability that allows you to exile the card from your hand with the specified number of time counters (the number before the dash) on it by paying its suspend cost (listed after the dash). The second is a triggered ability that removes a time counter from the suspended card at the beginning of each of your upkeeps. The third is a triggered ability that gives you the option to cast the card when the last time counter is removed."
    );
    supported("Search for Tomorrow");
    // Search for Tomorrow ({2}{G}): "Search your library for a basic land card, put it
    // onto the battlefield, then shuffle. Suspend 2—{G}"
    let mut t = TestGame::new(2);
    let forest = t.lands(P0, "Forest", 1)[0];
    let card = t.hand(P0, "Search for Tomorrow");
    // The static ability: pay {G} and exile it from the hand with two time counters.
    let exiled = suspend(&mut t, P0, card);
    assert!(t.obj(forest).tapped);
    assert_eq!(t.counters(exiled, counters::TIME), 2);
    assert_eq!(t.stack_len(), 0);
    // The first triggered ability: at the beginning of each of P0's upkeeps, remove a
    // time counter.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "Suspend"), 1);
    t.resolve();
    assert_eq!(t.counters(exiled, counters::TIME), 1);
    assert_eq!(t.stack_len(), 0);
    // The last one removed: the second triggered ability lets P0 cast it without paying
    // its mana cost.
    // (A Plains at the bottom of the library, so it isn't drawn in the meantime.)
    let plains = t.library_top(P0, "Plains");
    let lib = &mut t.g.players[P0.idx()].library;
    lib.pop();
    lib.insert(0, plains);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(plains)]);
    next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(t.counters(exiled, counters::TIME), 0);
    assert_eq!(t.stack_len(), 1);
    let lands_before = t.g.permanents().filter(|o| o.controller == P0).count();
    t.resolve();
    assert!(t.obj_now(card).is_spell());
    assert_eq!(t.g.history.spells_cast.len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Search for Tomorrow"));
    assert!(!t.named_on_battlefield("Plains").is_empty());
    assert_eq!(
        t.g.permanents().filter(|o| o.controller == P0).count(),
        lands_before + 1
    );
}

#[test]
fn an_alternative_cost_equal_to_a_missing_mana_cost_cant_be_paid() {
    cr!("118.6", "702.34a");
    ruling!(
        "Ancestral Vision",
        "If a card with no mana cost is given an alternative cost equal to its mana cost (by Snapcaster Mage, for example), that cost cannot be paid and the card cannot be cast this way."
    );
    supported("Ancestral Vision");
    supported("Snapcaster Mage");
    // Snapcaster Mage: "When this creature enters, target instant or sorcery card in your
    // graveyard gains flashback until end of turn. The flashback cost is equal to its mana
    // cost."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let vision = t.graveyard(P0, "Ancestral Vision");
    t.answer_targets(P0, &[Entity::Object(vision)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    assert!(t.obj_now(vision).chars.has_keyword(KeywordKind::Flashback));
    // Ancestral Vision has no mana cost: its flashback cost can't be paid.
    assert!(!can_cast(&mut t, P0, vision, FLASHBACK));
    let hand = t.hand_size(P0);
    assert!(t.cast(P0, vision).method(FLASHBACK).target(P0).try_go().is_err());
    t.clear_answers();
    assert!(t.in_graveyard(P0, "Ancestral Vision"));
    assert_eq!(t.hand_size(P0), hand);

    // A card with a mana cost can be cast this way ({1}{U} for Think Twice).
    let tt = t.graveyard(P0, "Think Twice");
    t.answer_targets(P0, &[Entity::Object(tt)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    assert!(can_cast(&mut t, P0, tt, FLASHBACK));
}

#[test]
fn granted_harmonize_and_madness_costs_equal_to_a_missing_mana_cost_cant_be_paid() {
    cr!("118.6", "702.180a", "702.35a");
    const HARMONIZE: CastMethod = CastMethod::Keyword(KeywordKind::Harmonize);
    supported("Songcrafter Mage");
    supported("Falkenrath Gorger");
    // Songcrafter Mage: "When this creature enters, target instant or sorcery card in your
    // graveyard gains harmonize until end of turn. Its harmonize cost is equal to its mana
    // cost."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let vision = t.graveyard(P0, "Ancestral Vision");
    t.answer_targets(P0, &[Entity::Object(vision)]);
    t.enter(P0, "Songcrafter Mage");
    t.resolve_all();
    assert!(t.obj_now(vision).chars.has_keyword(KeywordKind::Harmonize));
    assert!(!can_cast(&mut t, P0, vision, HARMONIZE));
    assert!(t
        .cast(P0, vision)
        .method(HARMONIZE)
        .target(P0)
        .try_go()
        .is_err());
    t.clear_answers();
    assert!(t.in_graveyard(P0, "Ancestral Vision"));

    // Falkenrath Gorger: "Each Vampire creature card you own that isn't on the battlefield
    // has madness. The madness cost is equal to its mana cost." A Vampire card with no
    // mana cost can't be cast for its madness cost: it's put into the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Falkenrath Gorger");
    t.lands(P0, "Swamp", 5);
    let vamp = custom_card("Costless Vampire", "Creature — Vampire", "", Some((2, 2)), "");
    let v = t.custom(P0, vamp, Zone::Hand(P0));
    assert!(t.obj_now(v).chars.has_keyword(KeywordKind::Madness));
    t.g.discard(P0, v, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(v));
    assert_eq!(t.zone(v), Zone::Graveyard(P0));
    assert!(t.g.permanents().all(|o| !o.tapped));
}
