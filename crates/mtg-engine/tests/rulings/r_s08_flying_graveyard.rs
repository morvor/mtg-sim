//! Rulings batch S08 — flying: Mistmoon Griffin ("When this creature dies, exile it, then
//! return the top creature card of your graveyard to the battlefield.") and Circling
//! Vultures ("At the beginning of your upkeep, sacrifice this creature unless you exile
//! the top creature card of your graveyard."): the order of a graveyard (CR 404.2, 404.3).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

const GRIFFIN: &str = "Mistmoon Griffin";

#[test]
fn the_top_card_is_the_one_put_there_most_recently() {
    cr!("404.2");
    ruling!(
        "Mistmoon Griffin",
        "The \"top\" card of your graveyard is the card that was put there most recently."
    );
    supported(GRIFFIN);
    for (first, second) in [("Grizzly Bears", "Hill Giant"), ("Hill Giant", "Grizzly Bears")] {
        let mut t = TestGame::new(2);
        let griffin = t.battlefield(P0, GRIFFIN);
        t.graveyard(P0, first);
        t.graveyard(P0, second);
        // A noncreature card on top doesn't matter: it's the top creature card.
        t.graveyard(P0, "Lightning Bolt");
        destroy(&mut t, griffin);
        t.resolve_all();
        assert!(t.in_exile(GRIFFIN));
        assert_eq!(t.named_on_battlefield(second).len(), 1, "{first}, {second}");
        assert!(t.in_graveyard(P0, first));
    }
}

#[test]
fn a_resolving_sorcery_goes_to_the_graveyard_last_on_top_of_what_it_destroyed() {
    cr!("608.2n", "404.3");
    ruling!(
        "Mistmoon Griffin",
        "The last thing that happens to a resolving instant or sorcery spell is that it's put into its owner's graveyard. —Example: You cast Wrath of God."
    );
    supported("Wrath of God");
    // P0's Wrath of God destroys the Griffin, Grizzly Bears and Hill Giant. P0 arranges
    // the cards; Wrath of God goes on top of them; the Griffin's trigger then returns the
    // top creature card (the Griffin itself is exiled first).
    let mut returned = Vec::new();
    for order in [vec![0, 1, 2], vec![2, 1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, GRIFFIN);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Hill Giant");
        t.lands(P0, "Plains", 4);
        let wrath = t.hand(P0, "Wrath of God");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        t.cast(P0, wrath).go();
        t.resolve();
        let gy = graveyard_names(&t, P0);
        assert_eq!(gy.len(), 4);
        assert_eq!(gy.last().map(String::as_str), Some("Wrath of God"));
        // The top creature card other than the Griffin.
        let expected = gy
            .iter()
            .rev()
            .find(|n| *n != "Wrath of God" && *n != GRIFFIN)
            .unwrap()
            .clone();
        t.resolve_all();
        assert!(t.in_exile(GRIFFIN));
        assert_eq!(t.named_on_battlefield(&expected).len(), 1, "{order:?}");
        returned.push(expected);
    }
    // The arrangement decided which card was on top.
    assert_ne!(returned[0], returned[1]);
}

#[test]
fn only_the_top_creature_card_can_be_exiled_and_the_order_stays() {
    cr!("404.2", "118.12");
    ruling!(
        "Circling Vultures",
        "Players may not rearrange the cards in their graveyards."
    );
    supported("Circling Vultures");
    // P0's graveyard, bottom to top: Hill Giant, Grizzly Bears, Lightning Bolt.
    let mut t = TestGame::new(2);
    let vultures = t.battlefield(P0, "Circling Vultures");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    let before = graveyard_names(&t, P0);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    let from = t.asked().len();
    t.resolve_all();
    // Only the Bears (the top creature card) could be exiled; the Vultures stay.
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    for c in &offered {
        assert!(!c.contains(&Entity::Object(giant)), "{c:?}");
    }
    assert!(!t.g.is_live(bears));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(vultures));
    // The rest of the graveyard keeps its order.
    let after = graveyard_names(&t, P0);
    let expected: Vec<String> = before
        .iter()
        .filter(|n| *n != "Grizzly Bears")
        .cloned()
        .collect();
    assert_eq!(after, expected);
    // Next upkeep, the Hill Giant is the top creature card; declining sacrifices it.
    t.answer_yes(P0, false);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Circling Vultures"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}
