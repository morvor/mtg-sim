//! Rulings batch S04 — descend N (ability words, CR 207.2c): abilities that care whether
//! there are at least N permanent cards in your graveyard. Permanent cards are artifact,
//! battle, creature, enchantment, land, and planeswalker cards (CR 110.4).

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts real cards into `p`'s graveyard.
fn bury(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.graveyard(p, n)).collect()
}

#[test]
fn descend_abilities_count_permanent_cards_in_your_graveyard() {
    cr!("207.2c", "110.4");
    ruling!(
        "Akawalli, the Seething Tower",
        "Cards with the ability word \"descend N\" have abilities that care if you have at least N permanent cards in your graveyard."
    );
    supported("Akawalli, the Seething Tower");
    // Akawalli: "Descend 4 — As long as there are four or more permanent cards in your
    // graveyard, Akawalli gets +2/+2 and has trample. Descend 8 — As long as there are
    // eight or more permanent cards in your graveyard, Akawalli gets an additional +2/+2
    // and can't be blocked by more than one creature."
    let mut t = TestGame::new(2);
    let akawalli = t.battlefield(P0, "Akawalli, the Seething Tower");
    // Three permanent cards and any number of instants and sorceries: not enough.
    bury(
        &mut t,
        P0,
        &[
            "Forest",
            "Grizzly Bears",
            "Mind Stone",
            "Lightning Bolt",
            "Shock",
            "Divination",
            "Cancel",
        ],
    );
    t.g.recompute();
    assert_eq!(t.pt(akawalli), (3, 3));
    assert!(!t.obj_now(akawalli).has_keyword(KeywordKind::Trample));
    // A fourth permanent card (an enchantment).
    bury(&mut t, P0, &["Fervor"]);
    t.g.recompute();
    assert_eq!(t.pt(akawalli), (5, 5));
    assert!(t.obj_now(akawalli).has_keyword(KeywordKind::Trample));
    // Eight: planeswalker, battle, and more.
    bury(
        &mut t,
        P0,
        &["Jace Beleren", "Invasion of Tarkir", "Hill Giant", "Swamp"],
    );
    t.g.recompute();
    assert_eq!(t.pt(akawalli), (7, 7));

    // Join the Dead: "Target creature gets -5/-5 until end of turn. Descend 4 — That
    // creature gets -10/-10 until end of turn instead if there are four or more permanent
    // cards in your graveyard."
    supported("Join the Dead");
    // Three permanent cards: an 8/6 (Craw Wurm with two +1/+1 counters) gets -5/-5.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(wurm), "+1/+1", 2, None);
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone", "Shock"]);
    give_mana_for(&mut t, P0, "Join the Dead");
    let join = t.hand(P0, "Join the Dead");
    t.cast(P0, join).target(wurm).go();
    t.resolve();
    assert_eq!(t.pt(wurm), (3, 1));
    // Four: it gets -10/-10 and dies.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(wurm), "+1/+1", 2, None);
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone", "Fervor"]);
    give_mana_for(&mut t, P0, "Join the Dead");
    let join = t.hand(P0, "Join the Dead");
    t.cast(P0, join).target(wurm).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn a_descend_trigger_checks_as_it_triggers_and_again_as_it_resolves() {
    cr!("603.4", "207.2c");
    // The ruling is shared by the descend cards (Terror Tide has it); Stinging Cave
    // Crawler's attack trigger is such an ability: "Descend 4 — Whenever this creature
    // attacks, if there are four or more permanent cards in your graveyard, you draw a
    // card and you lose 1 life."
    ruling!(
        "Terror Tide",
        "Some descend triggered abilities include intervening \"if\" clauses (i.e. \"if you have [four or eight] permanent cards in your graveyard\" in the middle of the ability). Each of these abilities checks your graveyard at the moment it would trigger to see if it does. If you don't have the required number of permanent cards in your graveyard at that time, the ability doesn't trigger at all. If it does trigger, it will check again as it tries to resolve. If you don't have the required number of permanent cards in your graveyard at that time, the ability won't resolve and none of its effects will happen."
    );
    supported("Stinging Cave Crawler");
    supported("Terror Tide");
    // Three permanent cards: it doesn't trigger.
    let mut t = TestGame::new(2);
    let crawler = t.battlefield(P0, "Stinging Cave Crawler");
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone", "Lightning Bolt"]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(crawler, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "permanent cards"), 0, "{:?}", stack_items(&t));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand, 20));

    // Four: it triggers and resolves.
    let mut t = TestGame::new(2);
    let crawler = t.battlefield(P0, "Stinging Cave Crawler");
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone", "Fervor"]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(crawler, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "permanent cards"), 1, "{:?}", stack_items(&t));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));

    // Four as it triggers, three as it resolves (Relic of Progenitus exiles one in
    // response): none of its effects happen.
    let mut t = TestGame::new(2);
    let crawler = t.battlefield(P0, "Stinging Cave Crawler");
    let relic = t.battlefield(P1, "Relic of Progenitus");
    let cards = bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone", "Fervor"]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(crawler, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "permanent cards"), 1);
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.activate(P1, relic, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand, 20));
}
