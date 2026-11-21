//! CR 701.51: open an Attraction; CR 701.52: roll to visit your Attractions.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{KeywordAction, Sel};
use mtg_engine::kwa::attractions::{attraction_deck, OPENED};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::variants::ROLLED_TO_VISIT;
use mtg_engine::*;

/// Puts Attraction cards into `p`'s Attraction deck (face down in the command zone), top
/// first.
fn attraction_deck_of(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = names
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

fn attractions(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Attraction"))
        .map(|o| o.chars.name.to_string())
        .collect()
}

/// Seasoned Buttoneer: "When this creature enters, open an Attraction."
fn buttoneer(t: &mut TestGame, p: PlayerId) {
    supported("Seasoned Buttoneer");
    t.enter(p, "Seasoned Buttoneer");
    t.resolve_all();
}

#[test]
fn only_a_player_with_an_attraction_deck_opens_attractions() {
    cr!("701.51a");
    let mut t = TestGame::new(2);
    attraction_deck_of(&mut t, P1, &["Information Booth"]);
    buttoneer(&mut t, P0);
    assert!(attractions(&t, P0).is_empty());
    assert!(custom_events(&t, OPENED).is_empty());
    // An opponent's Attraction deck isn't used.
    assert_eq!(attraction_deck(&t.g, P1).len(), 1);
}

#[test]
fn opening_an_attraction_puts_the_top_card_of_the_attraction_deck_onto_the_battlefield() {
    cr!("701.51b");
    supported("Information Booth");
    let mut t = TestGame::new(2);
    let deck = attraction_deck_of(&mut t, P0, &["Information Booth", "Kiddie Coaster"]);
    buttoneer(&mut t, P0);
    assert_eq!(attractions(&t, P0), vec!["Information Booth"]);
    let booth = t.g.current(deck[0]);
    assert_eq!(t.zone(booth), Zone::Battlefield);
    assert!(!t.obj_now(booth).face_down);
    assert_eq!(t.obj_now(booth).controller, P0);
    assert_eq!(attraction_deck(&t.g, P0), vec![deck[1]]);
    // Again: the next one; then the deck is empty and nothing happens.
    buttoneer(&mut t, P0);
    buttoneer(&mut t, P0);
    assert_eq!(attractions(&t, P0).len(), 2);
    assert!(attraction_deck(&t.g, P0).is_empty());
    assert_eq!(custom_events(&t, OPENED).len(), 2);
}

#[test]
fn whenever_you_open_an_attraction_triggers_only_if_it_entered() {
    cr!("701.51c");
    // The Most Dangerous Gamer: "Whenever you open an Attraction, put a +1/+1 counter on
    // The Most Dangerous Gamer."
    let mut t = TestGame::new(2);
    let gamer = t.battlefield(P0, "The Most Dangerous Gamer");
    attraction_deck_of(&mut t, P0, &["Information Booth", "Kiddie Coaster"]);
    buttoneer(&mut t, P0);
    assert_eq!(t.counters(gamer, counters::PLUS1), 1);
    // An effect stops it from entering: no trigger.
    t.custom(
        P1,
        text_card(
            "Closed for Repairs",
            "Enchantment",
            "{2}",
            None,
            "Artifact cards can't enter the battlefield.",
        ),
        Zone::Battlefield,
    );
    buttoneer(&mut t, P0);
    assert_eq!(attractions(&t, P0), vec!["Information Booth"]);
    assert_eq!(t.counters(gamer, counters::PLUS1), 1);
    assert_eq!(custom_events(&t, OPENED).len(), 1);
}

#[test]
fn rolling_to_visit_visits_the_attractions_with_the_result_lit_up() {
    cr!("701.52a");
    supported("Information Booth");
    supported("Kiddie Coaster");
    // Information Booth (2, 6 lit up): "Visit — Draw a card." Kiddie Coaster (2, 3, 6):
    // "Visit — Creatures you control get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Information Booth");
    t.battlefield(P0, "Kiddie Coaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let roll = |t: &mut TestGame, n: u32| {
        t.g.dice.loaded.push_back(n);
        run(t, P0, None, ka(KeywordAction::RollAttractions, Sel::None, 1), &[]);
        t.resolve_all();
    };
    let hand = t.hand_size(P0);
    roll(&mut t, 3);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.pt(bears), (3, 2));
    roll(&mut t, 2);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(bears), (4, 2));
    roll(&mut t, 5);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(bears), (4, 2));
    let results: Vec<i32> = custom_events(&t, ROLLED_TO_VISIT)
        .into_iter()
        .map(|(_, _, n)| n)
        .collect();
    assert_eq!(results, vec![3, 2, 5]);
    // An opponent's Attractions aren't visited by P0's roll.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Information Booth");
    let hand = t.hand_size(P1);
    t.g.dice.loaded.push_back(2);
    run(&mut t, P0, None, ka(KeywordAction::RollAttractions, Sel::None, 1), &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
}
