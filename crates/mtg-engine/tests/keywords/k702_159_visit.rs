//! CR 702.159 Visit (see also `tests/cr/r717_attraction_cards.rs`).

use crate::common_k702_153_167::*;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts Attraction cards into `p`'s Attraction deck (face down in the command zone).
fn attraction_deck_of(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let ids = names
        .iter()
        .map(|n| {
            let id = t.command(p, n);
            t.g.objects[id.0 as usize].face_down = true;
            id
        })
        .collect();
    t.g.recompute();
    ids
}

/// `p` rolls to visit their Attractions with the given die result.
fn roll_to_visit(t: &mut TestGame, p: PlayerId, result: u32) {
    t.g.dice.loaded.push_back(result);
    mtg_engine::variants::roll_to_visit(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn visit_triggers_when_the_roll_matches_a_lit_up_number() {
    cr!("702.159", "702.159a");
    // Information Booth (2 and 6 lit up): "Visit — Draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Information Booth");
    roll_to_visit(&mut t, P0, 6);
    assert_eq!(triggers_starting(&t, "Visit").len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // A result that isn't lit up: no visit.
    roll_to_visit(&mut t, P0, 4);
    assert!(triggers_starting(&t, "Visit").is_empty());
    // Only "you" rolling to visit: another player's roll doesn't visit this Attraction.
    roll_to_visit(&mut t, P1, 2);
    assert!(triggers_starting(&t, "Visit").is_empty());
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn claiming_the_prize_performs_the_prize_paragraph() {
    cr!("702.159b");
    assert_supported("Pick-a-Beeble");
    assert_supported("The Most Dangerous Gamer");
    // Pick-a-Beeble (2, 3, 6 lit up): "Visit — Roll a six-sided die. Put a number of luck
    // counters on this Attraction equal to the result and create a Treasure token. Then if
    // there are six or more luck counters on it, claim the prize!" "Prize — Create two
    // Treasure tokens, then sacrifice this Attraction and open an Attraction."
    let mut t = TestGame::new(2);
    let deck = attraction_deck_of(&mut t, P0, &["Information Booth"]);
    let beeble = t.battlefield(P0, "Pick-a-Beeble");
    // The Most Dangerous Gamer: "Whenever you claim the prize of an Attraction, destroy
    // target permanent."
    t.battlefield(P0, "The Most Dangerous Gamer");
    let victim = t.battlefield(P1, "Grizzly Bears");
    // First visit: a 5, no prize.
    roll_to_visit(&mut t, P0, 3);
    t.g.dice.loaded.push_back(5);
    t.resolve_all();
    assert_eq!(t.counters(beeble, "luck"), 5);
    assert_eq!(treasures(&t, P0), 1);
    assert!(t.on_battlefield(beeble));
    let claimed = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Custom { name, .. } if name == "claimed the prize"))
            .count()
    };
    assert_eq!(claimed(&t), 0);
    // Second visit: six or more luck counters: claim the prize.
    roll_to_visit(&mut t, P0, 6);
    t.g.dice.loaded.push_back(2);
    t.answer_targets(P0, &[Entity::Object(victim)]);
    t.resolve_all();
    assert_eq!(claimed(&t), 1);
    // One Treasure from the visit, two from the prize.
    assert_eq!(treasures(&t, P0), 4);
    // Sacrificed (into the junkyard) and an Attraction was opened.
    assert!(!t.on_battlefield(beeble));
    assert_eq!(t.zone(beeble), Zone::Command);
    assert!(t.on_battlefield(deck[0]));
    // "Whenever you claim the prize of an Attraction" triggered.
    assert!(!t.on_battlefield(victim));
}
