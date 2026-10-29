//! CR 702.38 Amplify.

use crate::common_k702_011_017::{assert_supported, custom_card};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;
use std::sync::Arc;

/// The candidates of each amplify reveal asked so far.
fn reveal_choices(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("amplify") => Some(candidates),
            _ => None,
        })
        .collect()
}

fn reveal(t: &mut TestGame, p: PlayerId, cards: &[ObjectId]) {
    let v: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    t.answer(p, DecisionKind::Entities, Answer::Entities(v));
}

#[test]
fn amplify_adds_n_counters_for_each_revealed_card_sharing_a_creature_type() {
    cr!("702.38", "702.38a");
    assert_supported("Glowering Rogon");
    let mut t = TestGame::new(2);
    // Glowering Rogon: 4/4 Beast, amplify 1.
    t.lands(P0, "Forest", 6);
    let rogon = t.hand(P0, "Glowering Rogon");
    let crawler = t.hand(P0, "Canopy Crawler"); // Beast
    let throwback = t.hand(P0, "Feral Throwback"); // Beast
    let bears = t.hand(P0, "Grizzly Bears"); // Bear
    reveal(&mut t, P0, &[crawler, throwback]);
    t.cast(P0, rogon).go();
    t.resolve_all();
    let rogon = t.named_on_battlefield("Glowering Rogon")[0];
    assert_eq!(t.counters(rogon, counters::PLUS1), 2);
    assert_eq!(t.pt(rogon), (6, 6));
    let cands = &reveal_choices(&t)[0];
    assert!(cands.contains(&Entity::Object(crawler)));
    assert!(cands.contains(&Entity::Object(throwback)));
    assert!(!cands.contains(&Entity::Object(bears)));
    // Revealed cards stay in hand.
    assert!(t.in_hand(P0, "Canopy Crawler") && t.in_hand(P0, "Feral Throwback"));
}

#[test]
fn any_number_of_cards_may_be_revealed() {
    cr!("702.38a");
    let mut t = TestGame::new(2);
    // Feral Throwback: 3/3 Beast, amplify 2. Reveal one of two Beasts.
    t.lands(P0, "Forest", 6);
    let throwback = t.hand(P0, "Feral Throwback");
    let crawler = t.hand(P0, "Canopy Crawler");
    t.hand(P0, "Glowering Rogon");
    reveal(&mut t, P0, &[crawler]);
    t.cast(P0, throwback).go();
    t.resolve_all();
    let tb = t.named_on_battlefield("Feral Throwback")[0];
    assert_eq!(t.counters(tb, counters::PLUS1), 2);
    // Revealing none is allowed too.
    let rogon = t.g.find_in_zone(Zone::Hand(P0), "Glowering Rogon")[0];
    t.lands(P0, "Forest", 6);
    reveal(&mut t, P0, &[]);
    t.cast(P0, rogon).go();
    t.resolve_all();
    let r = t.named_on_battlefield("Glowering Rogon")[0];
    assert_eq!(t.counters(r, counters::PLUS1), 0);
}

#[test]
fn amplify_applies_however_the_permanent_enters() {
    cr!("702.38a");
    let mut t = TestGame::new(2);
    t.hand(P0, "Canopy Crawler");
    let rogon = t.graveyard(P0, "Glowering Rogon");
    t.g.move_object(rogon, Zone::Battlefield, MoveCause::Effect, Some(P0));
    let r = t.named_on_battlefield("Glowering Rogon")[0];
    // By default every eligible card is revealed.
    assert_eq!(t.counters(r, counters::PLUS1), 1);
}

#[test]
fn cards_entering_at_the_same_time_cant_be_revealed() {
    cr!("702.38a");
    let mut t = TestGame::new(2);
    let rogon = t.hand(P0, "Glowering Rogon");
    let crawler = t.hand(P0, "Canopy Crawler");
    let throwback = t.hand(P0, "Feral Throwback");
    // Both Beasts enter from the hand at the same time.
    let ev = |obj| MoveEv {
        obj,
        to: Zone::Battlefield,
        pos: ability::LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(P0),
        etb: EtbInfo {
            controller: Some(P0),
            ..Default::default()
        },
        source: None,
    };
    t.g.move_objects(vec![ev(rogon), ev(crawler)]);
    for cands in reveal_choices(&t) {
        assert_eq!(cands, vec![Entity::Object(throwback)]);
    }
    let r = t.named_on_battlefield("Glowering Rogon")[0];
    let c = t.named_on_battlefield("Canopy Crawler")[0];
    assert_eq!(t.counters(r, counters::PLUS1), 1);
    assert_eq!(t.counters(c, counters::PLUS1), 1);
}

#[test]
fn a_changeling_card_shares_a_creature_type() {
    cr!("702.38a");
    let mut t = TestGame::new(2);
    // Changeling Outcast is every creature type.
    t.hand(P0, "Changeling Outcast");
    let rogon = t.graveyard(P0, "Glowering Rogon");
    t.g.move_object(rogon, Zone::Battlefield, MoveCause::Effect, Some(P0));
    let r = t.named_on_battlefield("Glowering Rogon")[0];
    assert_eq!(t.counters(r, counters::PLUS1), 1);
}

#[test]
fn each_instance_of_amplify_works_separately() {
    cr!("702.38b");
    let def = custom_card(
        "Doubly Amplified Beast",
        "Creature — Beast",
        Some((1, 1)),
        "Amplify 1\nAmplify 2",
    );
    let mut t = TestGame::new(2);
    let crawler = t.hand(P0, "Canopy Crawler");
    // The same card is revealed for each instance.
    reveal(&mut t, P0, &[crawler]);
    reveal(&mut t, P0, &[crawler]);
    let b = t.g.create_card_object(Arc::new(def), P0, Zone::Nowhere);
    t.g.move_object(b, Zone::Battlefield, MoveCause::Effect, Some(P0));
    let b = t.g.current(b);
    assert_eq!(reveal_choices(&t).len(), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 3);
}
