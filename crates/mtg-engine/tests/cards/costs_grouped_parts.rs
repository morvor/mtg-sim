//! Cost parts whose objects must be chosen together with a relationship (CR 118.3,
//! 602.2b): "Discard two nonland cards with the same name" (CR 201.2a), "Tap two untapped
//! creatures you control that share a creature type", "Reveal two cards from your hand
//! that share a color" (CR 105.2), "Tap any number of untapped creatures you control
//! other than ~ with total power 10 or greater", "Exile any number of historic cards from
//! your graveyard with total mana value 30 or greater"; and other cost parts: "Exile the
//! top creature card of your graveyard" (CR 404.2), "Reveal ~ from your hand" (CR 113.6,
//! 701.20a), "Remove X counters from an artifact or creature you control".

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn sphinx_of_the_chimes_discards_two_nonland_cards_with_the_same_name() {
    cr!("201.2a", "118.3");
    assert_supported("Sphinx of the Chimes");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx of the Chimes");
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    // A Lightning Bolt and a Shock don't have the same name; two Islands are lands.
    assert!(t.activate(P0, sphinx, 0, &[]).is_err());
    assert_eq!(t.hand_size(P0), 4);
    // With a second Lightning Bolt, the two Bolts can be discarded (an answer naming a
    // Bolt and the Shock is fitted to a pair with the same name).
    let bolt2 = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(shock)]);
    t.activate(P0, sphinx, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(!t.g.player(P0).hand.contains(&bolt) && !t.g.player(P0).hand.contains(&bolt2));
    assert!(t.in_hand(P0, "Shock"));
    t.resolve();
    // Shock, two Islands and four cards drawn.
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn weight_of_conscience_taps_two_creatures_that_share_a_creature_type() {
    cr!("118.3", "302.6");
    ruling!("Weight of Conscience", "you can tap creatures that haven’t been under your control");
    ruling!("Weight of Conscience", "The Aura will then be put into your graveyard");
    assert_supported("Weight of Conscience");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let woc = t.battlefield(P0, "Weight of Conscience");
    t.g.attach(woc, Entity::Object(giant));
    // A Bear and a Goblin share no creature type.
    let bear = t.battlefield_sick(P0, "Grizzly Bears");
    t.battlefield_sick(P0, "Goblin Piker");
    assert!(t.activate(P0, woc, 0, &[]).is_err());
    assert!(!t.g.obj(bear).tapped);
    // Two Bears do, even though they came under P0's control this turn ({T} isn't in
    // the cost).
    let bear2 = t.battlefield_sick(P0, "Grizzly Bears");
    t.activate(P0, woc, 0, &[]).unwrap();
    assert!(t.g.obj(bear).tapped && t.g.obj(bear2).tapped);
    t.resolve();
    assert!(t.in_exile("Hill Giant"));
    t.settle();
    assert!(t.in_graveyard(P0, "Weight of Conscience"));
}

#[test]
fn illuminated_folio_reveals_two_cards_that_share_a_color() {
    cr!("105.2", "701.20a");
    ruling!("Illuminated Folio", "Colorless is not a color");
    assert_supported("Illuminated Folio");
    let mut t = TestGame::new(2);
    let folio = t.battlefield(P0, "Illuminated Folio");
    t.lands(P0, "Plains", 1);
    t.hand(P0, "Sol Ring");
    t.hand(P0, "Mind Stone");
    t.hand(P0, "Shock");
    // Two colorless cards don't share a color; a red card shares none with them.
    assert!(t.activate(P0, folio, 0, &[]).is_err());
    t.hand(P0, "Lightning Bolt");
    t.activate(P0, folio, 0, &[]).unwrap();
    t.resolve();
    // Revealing doesn't move the cards: four in hand, plus the card drawn.
    assert_eq!(t.hand_size(P0), 5);
}

#[test]
fn mossbridge_troll_taps_creatures_with_total_power_ten_or_greater() {
    cr!("118.3", "602.2b");
    ruling!("Mossbridge Troll", "you may tap a Mossbridge Troll to help pay the cost of a different");
    assert_supported("Mossbridge Troll");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Mossbridge Troll");
    t.battlefield(P0, "Craw Wurm");
    // A Craw Wurm alone has power 6; the Troll can't tap itself for its own cost.
    assert!(t.activate(P0, troll, 0, &[]).is_err());
    // Another Mossbridge Troll (5) and the Craw Wurm (6) make 11.
    let other = t.battlefield(P0, "Mossbridge Troll");
    t.activate(P0, troll, 0, &[]).unwrap();
    assert!(t.g.obj(other).tapped);
    assert!(!t.g.obj(troll).tapped);
    t.resolve();
    assert_eq!(t.pt(troll), (25, 25));
}

#[test]
fn the_capitoline_triad_exiles_historic_cards_with_total_mana_value_30() {
    cr!("118.3", "107.3a");
    assert_supported("The Capitoline Triad");
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, "The Capitoline Triad");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Historic cards with total mana value 10 + 11 = 21: not enough.
    t.graveyard(P0, "Ulamog, the Ceaseless Hunger");
    t.graveyard(P0, "Darksteel Colossus");
    // A nonhistoric card doesn't count.
    t.graveyard(P0, "Craw Wurm");
    assert!(t.activate(P0, triad, 0, &[]).is_err());
    assert_eq!(t.graveyard_size(P0), 3);
    // With Kozilek, Butcher of Truth (10) there's 31.
    t.graveyard(P0, "Kozilek, Butcher of Truth");
    t.activate(P0, triad, 0, &[]).unwrap();
    // The cards exiled are historic and have total mana value 30 or greater.
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    t.resolve();
    assert_eq!(t.pt(bears), (9, 9));
}

#[test]
fn zombie_scavengers_exiles_the_top_creature_card_of_its_graveyard() {
    cr!("404.2", "701.19a");
    ruling!("Zombie Scavengers", "is the card that was put there most recently");
    assert_supported("Zombie Scavengers");
    assert_supported("Necratog");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zombie Scavengers");
    // No creature card in the graveyard.
    t.graveyard(P0, "Island");
    assert!(t.activate(P0, z, 0, &[]).is_err());
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Swamp");
    t.activate(P0, z, 0, &[]).unwrap();
    // The Hill Giant was the creature card put there most recently.
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve();
    let z = t.g.current(z);
    t.g.destroy(z, None);
    t.settle();
    assert!(t.on_battlefield(z));
}

#[test]
fn tetzimoc_reveals_itself_from_hand_during_your_turn() {
    cr!("113.6", "701.20a", "602.5d");
    ruling!("Tetzimoc, Primal Death", "more than once during your turn");
    assert_supported("Tetzimoc, Primal Death");
    let mut t = TestGame::new(2);
    let tetz = t.hand(P0, "Tetzimoc, Primal Death");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.activate(P0, tetz, 0, &[Entity::Object(a)]).unwrap();
    t.resolve();
    t.activate(P0, tetz, 0, &[Entity::Object(b)]).unwrap();
    t.resolve();
    assert_eq!(t.counters(a, "prey"), 1);
    assert_eq!(t.counters(b, "prey"), 1);
    // Still in hand (revealing doesn't move it).
    assert!(t.in_hand(P0, "Tetzimoc, Primal Death"));
    // Not during an opponent's turn.
    t.lands(P0, "Swamp", 1);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P0, tetz, 0, &[Entity::Object(a)]).is_err());
}

#[test]
fn moxite_refinery_removes_x_counters_from_one_permanent() {
    cr!("118.3", "107.3a");
    assert_supported("Moxite Refinery");
    let mut t = TestGame::new(2);
    let refinery = t.battlefield(P0, "Moxite Refinery");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let rock = t.battlefield(P0, "Mind Stone");
    t.g.add_counters(Entity::Object(a), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(b), "+1/+1", 1, None);
    t.lands(P0, "Plains", 2);
    // X = 3: no one permanent has three counters (three among them isn't enough).
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    assert!(t.activate(P0, refinery, 0, &[Entity::Object(rock)]).is_err());
    t.clear_answers();
    // X = 2, from the Bears: two charge counters on the Mind Stone.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, refinery, 0, &[Entity::Object(rock)]).unwrap();
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    t.resolve();
    assert_eq!(t.counters(rock, "charge"), 2);
}
