//! Qualifiers on trigger events (CR 603.2): "during your turn", "for the first time each
//! turn", "for the first time during each of your/their turns", "while [condition]"
//! (part of the event, checked as it happens; "while ~ is in your graveyard" functions
//! from the graveyard, CR 113.6), and "without being played".

use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn count_subtype(t: &TestGame, subtype: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype(subtype))
        .count()
}

#[test]
fn dies_during_your_turn() {
    cr!("603.2", "700.4");
    supported("Vogar, Necropolis Tyrant");
    let mut t = TestGame::new(2);
    let vogar = t.battlefield(P0, "Vogar, Necropolis Tyrant");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(t.counters(vogar, "+1/+1"), 1);
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    let other = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(other, None);
    t.resolve_all();
    assert_eq!(t.counters(vogar, "+1/+1"), 1);
}

#[test]
fn land_enters_during_your_turn() {
    cr!("603.2", "603.6a");
    supported("Foe-liage");
    let mut t = TestGame::new(2);
    let foe = t.battlefield(P0, "Foe-liage");
    t.enter(P1, "Forest");
    t.resolve_all();
    assert_eq!(t.counters(foe, "+1/+1"), 1, "any land, during your turn");
    t.set_step(P1, Step::PrecombatMain);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.counters(foe, "+1/+1"), 1);
}

#[test]
fn becomes_tapped_during_your_turn_once_each_turn() {
    cr!("603.2");
    supported("Interface Ace");
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Interface Ace");
    t.g.tap(ace);
    t.resolve_all();
    assert!(!t.obj_now(ace).tapped, "untapped by its ability");
    t.g.tap(ace);
    t.resolve_all();
    assert!(t.obj_now(ace).tapped, "triggers only once each turn");
    // During an opponent's turn: no.
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Interface Ace");
    t.set_step(P1, Step::PrecombatMain);
    t.g.tap(ace);
    t.resolve_all();
    assert!(t.obj_now(ace).tapped, "not during your turn");
}

#[test]
fn counters_put_on_it_for_the_first_time_each_turn() {
    cr!("603.2", "122.6");
    supported("Axgard Artisan");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Axgard Artisan");
    t.g.add_counters(Entity::Object(artisan), "+1/+1", 2, None);
    t.resolve_all();
    t.g.add_counters(Entity::Object(artisan), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Treasure"), 1);
}

#[test]
fn gain_life_for_the_first_time_during_each_of_your_turns() {
    cr!("603.2");
    supported("Cat Collector");
    ruling!(
        "Cat Collector",
        "before Cat Collector is on the battlefield"
    );
    let cats = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Cat"))
            .count()
    };
    let mut t = TestGame::new(2);
    let collector = t.battlefield(P0, "Cat Collector");
    let base = cats(&t);
    t.g.gain_life(P0, 2);
    t.resolve_all();
    t.g.gain_life(P0, 3);
    t.resolve_all();
    assert_eq!(cats(&t), base + 1, "one Cat for the first life gain");
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.number += 1;
    t.g.turn_events.clear();
    t.g.gain_life(P0, 2);
    t.resolve_all();
    assert_eq!(cats(&t), base + 1);
    let _ = collector;
    // Life gained this turn before it was on the battlefield counts as the first time.
    let mut t = TestGame::new(2);
    t.g.gain_life(P0, 1);
    t.resolve_all();
    t.battlefield(P0, "Cat Collector");
    let base = cats(&t);
    t.g.gain_life(P0, 1);
    t.resolve_all();
    assert_eq!(cats(&t), base);
}

#[test]
fn opponent_loses_life_for_the_first_time_during_each_of_their_turns() {
    cr!("603.2");
    supported("Valgavoth, Harrower of Souls");
    ruling!(
        "Valgavoth, Harrower of Souls",
        "had lost life earlier in the turn"
    );
    let mut t = TestGame::new(2);
    let valgavoth = t.battlefield(P0, "Valgavoth, Harrower of Souls");
    // During your own turn: no.
    t.g.lose_life(P1, 1);
    t.resolve_all();
    assert_eq!(t.counters(valgavoth, "+1/+1"), 0);
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn_events.clear();
    let hand = t.hand_size(P0);
    t.g.lose_life(P1, 1);
    t.resolve_all();
    t.g.lose_life(P1, 1);
    t.resolve_all();
    assert_eq!(t.counters(valgavoth, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // It enters during that opponent's turn after they lost life: their first loss this
    // turn is already past, so a later one doesn't trigger it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.g.lose_life(P1, 1);
    t.resolve_all();
    let valgavoth = t.battlefield(P0, "Valgavoth, Harrower of Souls");
    t.g.lose_life(P1, 1);
    t.resolve_all();
    assert_eq!(t.counters(valgavoth, "+1/+1"), 0);
    // (Without the earlier loss, the same loss triggers it.)
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let valgavoth = t.battlefield(P0, "Valgavoth, Harrower of Souls");
    t.g.lose_life(P1, 1);
    t.resolve_all();
    assert_eq!(t.counters(valgavoth, "+1/+1"), 1);
}

#[test]
fn discard_one_or_more_cards_for_the_first_time_each_turn() {
    cr!("603.2", "603.2c", "701.9a");
    supported("Rielle, the Everwise");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rielle, the Everwise");
    t.lands(P0, "Swamp", 3);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(Entity::Player(P0)).go();
    t.resolve_all();
    // Both cards were discarded at once: draw that many.
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 3);
    // Not a second time this turn.
    let c = t.g.player(P0).hand[0];
    t.g.discard(P0, c, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn dies_while_this_card_is_in_your_graveyard() {
    cr!("603.10a", "113.6");
    supported("Furious Forebear");
    ruling!(
        "Furious Forebear",
        "dies at the same time as one or more creatures you control"
    );
    let mut t = TestGame::new(2);
    let forebear = t.graveyard(P0, "Furious Forebear");
    t.lands(P0, "Plains", 2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(bear, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Furious Forebear"));
    let _ = forebear;
    // An opponent's creature: no.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Furious Forebear");
    t.lands(P0, "Plains", 2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Furious Forebear"));
    // Dying together with the creature: it wasn't in the graveyard yet.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Furious Forebear");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.answer_yes(P0, true);
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Furious Forebear"));
    assert!(!t.in_hand(P0, "Furious Forebear"));
}

#[test]
fn becomes_tapped_while_it_has_a_counter() {
    cr!("603.2");
    supported("Encumbered Reejerey");
    let mut t = TestGame::new(2);
    let reejerey = t.battlefield(P0, "Encumbered Reejerey");
    t.g.add_counters(Entity::Object(reejerey), "-1/-1", 1, None);
    t.g.tap(reejerey);
    t.resolve_all();
    assert_eq!(t.counters(reejerey, "-1/-1"), 0);
    // The qualifier is part of the event, checked as it becomes tapped: with no counter
    // then, nothing triggers, even if it has one before the ability would resolve.
    t.g.untap(reejerey);
    t.g.tap(reejerey);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.g.add_counters(Entity::Object(reejerey), "-1/-1", 1, None);
    t.resolve_all();
    assert_eq!(t.counters(reejerey, "-1/-1"), 1);
}

#[test]
fn another_creature_enters_while_this_has_a_counter() {
    cr!("603.2", "603.6a");
    supported("Bristlebane Battler");
    let mut t = TestGame::new(2);
    let battler = t.battlefield(P0, "Bristlebane Battler");
    t.g.add_counters(Entity::Object(battler), "-1/-1", 1, None);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(battler, "-1/-1"), 0);
    // No counter as the next one enters: nothing triggers (the qualifier is checked as
    // the event happens), even if it has a counter again before anything would resolve.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.g.add_counters(Entity::Object(battler), "-1/-1", 1, None);
    t.resolve_all();
    assert_eq!(t.counters(battler, "-1/-1"), 1);
}

#[test]
fn dies_or_is_exiled_while_its_power_is_four_or_greater() {
    cr!("603.10a", "603.6c");
    supported("Syr Vondam, Sunstar Exemplar");
    for (counters, destroyed) in [(0, false), (4, true)] {
        let mut t = TestGame::new(2);
        let vondam = t.battlefield(P0, "Syr Vondam, Sunstar Exemplar");
        let rock = t.battlefield(P1, "Sol Ring");
        if counters > 0 {
            t.g.add_counters(Entity::Object(vondam), "+1/+1", counters, None);
        }
        t.answer_targets(P0, &[Entity::Object(rock)]);
        t.g.exile_object(vondam, None);
        t.resolve_all();
        assert_eq!(
            !t.on_battlefield(rock),
            destroyed,
            "+1/+1 counters: {counters}"
        );
    }
}

#[test]
fn lands_enter_under_an_opponents_control_without_being_played() {
    cr!("603.2", "305.1");
    supported("Deep Gnome Terramancer");
    let plains_count = |t: &TestGame| t.named_on_battlefield("Plains").len();
    // Played: no.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deep Gnome Terramancer");
    t.library_top(P0, "Plains");
    t.set_step(P1, Step::PrecombatMain);
    let land = t.hand(P1, "Forest");
    t.answer_yes(P0, true);
    t.play_land(P1, land).unwrap();
    t.resolve_all();
    assert_eq!(plains_count(&t), 0);
    // Put onto the battlefield by an effect: search for a Plains.
    let forest = t.library_top(P1, "Forest");
    t.answer_yes(P0, true);
    t.g.move_object(forest, Zone::Battlefield, MoveCause::Effect, Some(P1));
    t.resolve_all();
    assert_eq!(plains_count(&t), 1);
}

#[test]
fn a_card_leaves_your_graveyard_during_your_turn() {
    cr!("603.10a");
    supported("Kishla Skimmer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kishla Skimmer");
    let a = t.graveyard(P0, "Island");
    let b = t.graveyard(P0, "Island");
    let hand = t.hand_size(P0);
    t.g.exile_object(a, None);
    t.resolve_all();
    t.g.exile_object(b, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "only once each turn");
    // Not during an opponent's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kishla Skimmer");
    let a = t.graveyard(P0, "Island");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.g.exile_object(a, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn sacrifice_a_permanent_during_your_turn() {
    cr!("603.2", "701.21a");
    supported("Tolls of War");
    let allies = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Ally"))
            .count()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tolls of War");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bear, P0);
    t.resolve_all();
    assert_eq!(allies(&t), 1);
    t.set_step(P1, Step::PrecombatMain);
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(other, P0);
    t.resolve_all();
    assert_eq!(allies(&t), 1);
}
