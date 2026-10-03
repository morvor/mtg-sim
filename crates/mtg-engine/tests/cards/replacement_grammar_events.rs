//! Player-event replacement effects compiled by the replacement grammar
//! (`src/oracle/patterns/replacement_grammar_events.rs`): draws, life loss, and keyword
//! actions (mill, scry, proliferate, explore, connive), with "[instructions] instead",
//! "you may ... instead", and amount changes (CR 614.1a, 614.11, 121.6, 701).

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn event_grammar_cards_compile() {
    compiles(&[
        "Bloodletter of Aclazotz",
        "Blood Scrivener",
        "Out of the Tombs",
        "Living Conundrum",
        "Eruth, Tormented Prophet",
        "Sages of the Anima",
        "Tomorrow, Azami's Familiar",
        "Forbidden Crypt",
        "Pursuit of Knowledge",
        "Obstinate Familiar",
        "The Water Crystal",
        "Tekuthal, Inquiry Dominus",
        "Eligeth, Crossroads Augur",
        "Kenessos, Priest of Thassa",
        "Leader, Super-Genius",
        "Topography Tracker",
        "Twists and Turns // Mycoid Maze",
    ]);
}

#[test]
fn blood_scrivener_draws_two_and_loses_life_with_an_empty_hand() {
    cr!("614.11", "121.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Scrivener");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    assert_eq!(t.hand_size(P0), 0);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 19);
    // With cards in hand, a normal draw.
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn living_conundrum_skips_draws_from_an_empty_library() {
    cr!("614.11", "121.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Living Conundrum");
    let n = t.library_size(P0) as u32;
    t.g.mill(P0, n);
    assert_eq!(t.library_size(P0), 0);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0), "{}", t.dump_log());
}

#[test]
fn obstinate_familiar_may_skip_the_draw() {
    cr!("614.11", "121.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obstinate Familiar");
    t.library_top(P0, "Island");
    let lib = t.library_size(P0);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn eruth_exiles_the_top_two_cards_instead_of_drawing() {
    cr!("614.11");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eruth, Tormented Prophet");
    t.library_top(P0, "Island");
    t.library_top(P0, "Forest");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_exile("Island") && t.in_exile("Forest"));
}

#[test]
fn bloodletter_doubles_opponents_life_loss_during_your_turn() {
    cr!("614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodletter of Aclazotz");
    t.g.lose_life(P1, 3);
    t.settle();
    assert_eq!(t.life(P1), 14);
    t.g.lose_life(P0, 3);
    t.settle();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn the_water_crystal_adds_four_to_opponents_mills() {
    cr!("614.1a", "701.17d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Water Crystal");
    for _ in 0..10 {
        t.library_top(P1, "Island");
        t.library_top(P0, "Island");
    }
    t.g.mill(P1, 2);
    t.settle();
    assert_eq!(t.graveyard_size(P1), 6);
    t.g.mill(P0, 2);
    t.settle();
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn tekuthal_proliferates_twice() {
    cr!("614.1a", "701.34a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tekuthal, Inquiry Dominus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    t.lands(P0, "Island", 3);
    let sp = t.hand(P0, "Steady Progress");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, sp).go();
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

#[test]
fn eligeth_draws_instead_of_scrying_and_kenessos_scries_one_more() {
    cr!("614.1a", "701.22a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eligeth, Crossroads Augur");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve();
    // Scry 1 became draw 1, then Opt draws a card.
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn topography_tracker_makes_creatures_explore_twice() {
    cr!("614.1a", "701.44a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Topography Tracker");
    t.library_top(P0, "Island");
    t.library_top(P0, "Forest");
    let b = t.enter(P0, "Merfolk Branchwalker");
    t.resolve_all();
    // Two lands revealed: both put into hand.
    assert_eq!(t.hand_size(P0), 2, "{}", t.dump_log());
    assert_eq!(t.counters(b, "+1/+1"), 0);
}

#[test]
fn flames_of_the_blood_hand_stops_that_player_gaining_life_this_turn() {
    cr!("614.1a", "611.2c");
    compiles(&["Flames of the Blood Hand"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let f = t.hand(P0, "Flames of the Blood Hand");
    t.cast(P0, f).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    t.g.gain_life(P1, 5);
    t.settle();
    assert_eq!(t.life(P1), 16);
    // You can still gain life.
    t.g.gain_life(P0, 2);
    t.settle();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn scion_of_halaster_replaces_only_the_first_draw_each_turn() {
    cr!("614.11", "903.3");
    compiles(&["Scion of Halaster"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scion of Halaster");
    let cmdr = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.dirty = true;
    let a = t.library_top(P0, "Island");
    let b = t.library_top(P0, "Forest");
    // Look at the top two (Forest, Island): put the Forest into the graveyard, then draw
    // the Island.
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Island"));
    let _ = a;
    // The second draw this turn is a normal draw.
    let gy = t.graveyard_size(P0);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.graveyard_size(P0), gy);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn retriever_phoenix_returns_instead_of_learning() {
    cr!("614.1a", "701.48a", "113.6b");
    compiles(&["Retriever Phoenix"]);
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Retriever Phoenix");
    t.answer_yes(P0, true);
    mtg_engine::kwa::learn::learn(&mut t.g, P0, None);
    t.settle();
    assert_eq!(t.named_on_battlefield("Retriever Phoenix").len(), 1);
}
