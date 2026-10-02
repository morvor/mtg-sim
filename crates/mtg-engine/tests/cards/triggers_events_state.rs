//! State triggers (CR 603.8) compiled from "When/Whenever [game state]" through the
//! condition grammar: counters on the source, life totals, hand sizes, what players
//! control, the source's keywords and its owner.

use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

#[test]
fn no_ice_counters_sacrifice_and_create_marit_lage() {
    cr!("603.8");
    supported("Dark Depths");
    let mut t = TestGame::new(2);
    let depths = t.battlefield(P0, "Dark Depths");
    t.g.add_counters(Entity::Object(depths), "ice", 1, None);
    t.resolve_all();
    assert!(t.on_battlefield(depths), "one ice counter left: no trigger");
    t.g.remove_counters(Entity::Object(depths), "ice", 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dark Depths"));
    assert_eq!(t.named_on_battlefield("Marit Lage").len(), 1);
}

#[test]
fn state_trigger_doesnt_retrigger_while_on_the_stack() {
    cr!("603.8");
    supported("Plague Boiler");
    let mut t = TestGame::new(2);
    let boiler = t.battlefield(P0, "Plague Boiler");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    t.g.add_counters(Entity::Object(boiler), "plague", 3, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The state still matches, but the ability is already on the stack.
    t.settle();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // "sacrifice it. If you do, destroy all nonland permanents."
    assert!(t.in_graveyard(P0, "Plague Boiler"));
    assert!(!t.on_battlefield(bear));
    assert!(t.on_battlefield(forest));
}

#[test]
fn counter_thresholds_on_the_source() {
    cr!("603.8");
    for name in ["Mazemind Tome", "Deadly Designs", "Darksteel Reactor"] {
        supported(name);
    }
    // "When there are four or more page counters on ~, exile it. If you do, you gain 4
    // life."
    let mut t = TestGame::new(2);
    let tome = t.battlefield(P0, "Mazemind Tome");
    t.g.add_counters(Entity::Object(tome), "page", 3, None);
    t.resolve_all();
    assert!(t.on_battlefield(tome));
    t.g.add_counters(Entity::Object(tome), "page", 1, None);
    t.resolve_all();
    assert!(t.in_exile("Mazemind Tome"));
    assert_eq!(t.life(P0), 24);
    // "When ~ has twenty or more charge counters on it, you win the game."
    let mut t = TestGame::new(2);
    let reactor = t.battlefield(P0, "Darksteel Reactor");
    t.g.add_counters(Entity::Object(reactor), "charge", 19, None);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    t.g.add_counters(Entity::Object(reactor), "charge", 1, None);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn no_counters_of_a_kind_left() {
    cr!("603.8");
    supported("Afiya Grove");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Afiya Grove");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Afiya Grove"));
    let mut t = TestGame::new(2);
    let grove = t.battlefield(P0, "Afiya Grove");
    t.g.add_counters(Entity::Object(grove), "+1/+1", 1, None);
    t.resolve_all();
    assert!(t.on_battlefield(grove));
}

#[test]
fn life_total_states_turn_enchantments_into_creatures() {
    cr!("603.8", "603.4");
    for name in ["Opal Avenger", "Lurking Jackals", "Transcendence"] {
        supported(name);
    }
    // "When you have 10 or less life, if ~ is an enchantment, it becomes a 3/5 Soldier
    // creature."
    let mut t = TestGame::new(2);
    let avenger = t.battlefield(P0, "Opal Avenger");
    t.resolve_all();
    assert!(!t.obj_now(avenger).is(CardType::Creature));
    t.g.lose_life(P0, 10);
    t.resolve_all();
    assert!(t.obj_now(avenger).is(CardType::Creature));
    assert_eq!(t.pt(avenger), (3, 5));
    // "When an opponent has 10 or less life, ..."
    let mut t = TestGame::new(2);
    let jackals = t.battlefield(P0, "Lurking Jackals");
    t.g.lose_life(P0, 15);
    t.resolve_all();
    assert!(!t.obj_now(jackals).is(CardType::Creature), "your own life");
    t.g.lose_life(P1, 10);
    t.resolve_all();
    assert_eq!(t.pt(jackals), (3, 2));
    // "When you have 20 or more life, you lose the game."
    let mut t = TestGame::new(2);
    t.g.lose_life(P0, 5);
    t.resolve_all();
    t.battlefield(P0, "Transcendence");
    t.resolve_all();
    assert!(!t.has_lost(P0));
    t.g.gain_life(P0, 5);
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn a_player_has_no_cards_in_hand() {
    cr!("603.8");
    supported("Veiled Crocodile");
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    let theirs = t.hand(P1, "Island");
    let croc = t.battlefield(P0, "Veiled Crocodile");
    t.resolve_all();
    assert!(!t.obj_now(croc).is(CardType::Creature));
    t.g.discard(P1, theirs, None);
    t.resolve_all();
    assert_eq!(t.pt(croc), (4, 4));
}

#[test]
fn what_players_control() {
    cr!("603.8");
    for name in [
        "Hidden Predators",
        "Endrek Sahr, Master Breeder",
        "Last Laugh",
    ] {
        supported(name);
    }
    // "When an opponent controls a creature with power 4 or greater, ..."
    let mut t = TestGame::new(2);
    let predators = t.battlefield(P0, "Hidden Predators");
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(!t.obj_now(predators).is(CardType::Creature));
    t.battlefield(P1, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.pt(predators), (4, 4));
    // "When you control seven or more Thrulls, sacrifice ~."
    let mut t = TestGame::new(2);
    let endrek = t.battlefield(P0, "Endrek Sahr, Master Breeder");
    for _ in 0..6 {
        t.battlefield(P0, "Basal Thrull");
    }
    t.resolve_all();
    assert!(t.on_battlefield(endrek));
    t.battlefield(P0, "Basal Thrull");
    t.resolve_all();
    assert!(!t.on_battlefield(endrek));
    // "When no creatures are on the battlefield, sacrifice ~."
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let laugh = t.battlefield(P0, "Last Laugh");
    t.resolve_all();
    assert!(t.on_battlefield(laugh));
    t.g.destroy(bear, None);
    t.resolve_all();
    assert!(!t.on_battlefield(laugh));
}

#[test]
fn when_this_has_flying() {
    cr!("603.8");
    supported("Floodgate");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Floodgate");
    t.resolve_all();
    assert!(t.on_battlefield(gate));
    // Levitation: "Creatures you control have flying."
    t.battlefield(P0, "Levitation");
    t.resolve_all();
    assert!(!t.on_battlefield(gate));
}
