//! Looking at and revealing the top cards of a library and splitting them up: "put one of
//! them into your hand and the rest on the bottom of your library in any order", "you may
//! reveal a creature card from among them and put it into your hand. Put the rest on the
//! bottom of your library in a random order.", "then put them back in any order", "put
//! the rest into your graveyard".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Candidates offered by the most recent "choose entities" decision.
fn last_choice_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("no choice was asked")
}

/// Stacks named cards on top of P0's library; the last one named ends up on top.
fn stack(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.library_top(P0, n)).collect()
}

fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.g.obj(id).chars.name.to_string()
}

/// P0's library, top first.
fn library_top_first(t: &TestGame) -> Vec<ObjectId> {
    t.g.player(P0).library.iter().rev().copied().collect()
}

#[test]
fn sleight_of_hand_one_to_hand_the_other_to_the_bottom() {
    cr!("701.20e");
    assert_supported("Sleight of Hand");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bottom_before = t.g.player(P0).library[0];
    let ids = stack(&mut t, &["Forest", "Lightning Bolt", "Grizzly Bears"]);
    let (bolt, bears) = (ids[1], ids[2]);
    let sleight = t.hand(P0, "Sleight of Hand");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, sleight).go();
    t.resolve();
    // Only the top two cards were offered.
    let offered = last_choice_candidates(&t);
    assert_eq!(offered.len(), 2);
    assert!(offered.contains(&Entity::Object(bears)) && offered.contains(&Entity::Object(bolt)));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // Grizzly Bears went to the bottom (below the old bottom card), the Forest is on top.
    let lib = &t.g.player(P0).library;
    assert_eq!(name_of(&t, lib[0]), "Grizzly Bears");
    assert_eq!(lib[1], bottom_before);
    assert_eq!(library_top_first(&t)[0], ids[0]);
}

#[test]
fn strategic_planning_rest_into_the_graveyard() {
    cr!("701.20e");
    assert_supported("Strategic Planning");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let ids = stack(&mut t, &["Forest", "Lightning Bolt", "Grizzly Bears", "Shock"]);
    let plan = t.hand(P0, "Strategic Planning");
    t.answer_choose(P0, &[Entity::Object(ids[2])]);
    t.cast(P0, plan).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Shock"));
    // The fourth card is untouched on top.
    assert_eq!(library_top_first(&t)[0], ids[0]);
}

#[test]
fn anticipate_rest_on_the_bottom_in_the_chosen_order() {
    cr!("701.20e", "401.4");
    assert_supported("Anticipate");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    // Top first: Shock, Grizzly Bears, Lightning Bolt.
    let ids = stack(&mut t, &["Lightning Bolt", "Grizzly Bears", "Shock"]);
    let bears = ids[1];
    let anticipate = t.hand(P0, "Anticipate");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    // The rest (top first: Shock, Lightning Bolt): put Lightning Bolt above Shock.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.cast(P0, anticipate).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let lib = &t.g.player(P0).library;
    assert_eq!(name_of(&t, lib[0]), "Shock", "Shock is at the very bottom");
    assert_eq!(name_of(&t, lib[1]), "Lightning Bolt");
}

#[test]
fn crystal_seer_puts_them_back_in_any_order() {
    cr!("701.20e", "401.4");
    assert_supported("Crystal Seer");
    let mut t = TestGame::new(2);
    // Top first: A, B, C, D (then fillers).
    stack(&mut t, &["Shock", "Lightning Bolt", "Grizzly Bears", "Forest"]);
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![3, 2, 1, 0]));
    t.enter(P0, "Crystal Seer");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Reversed: Shock is now on top.
    let top: Vec<String> = library_top_first(&t)[..4]
        .iter()
        .map(|c| name_of(&t, *c))
        .collect();
    assert_eq!(top, ["Shock", "Lightning Bolt", "Grizzly Bears", "Forest"]);
}

#[test]
fn vivien_reid_reveals_a_creature_or_land_card_rest_randomly_on_the_bottom() {
    cr!("701.20e");
    assert_supported("Vivien Reid");
    let mut t = TestGame::new(2);
    let vivien = t.battlefield(P0, "Vivien Reid");
    let ids = stack(
        &mut t,
        &["Shock", "Grizzly Bears", "Lightning Bolt", "Forest", "Duress"],
    );
    // Top first: Duress, Forest, Lightning Bolt, Grizzly Bears, then Shock.
    let (shock, bears, bolt, forest, duress) = (ids[0], ids[1], ids[2], ids[3], ids[4]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.activate(P0, vivien, 0, &[]).unwrap();
    t.resolve();
    // Only the creature and the land among the top four were offered.
    let mut offered = last_choice_candidates(&t);
    offered.sort();
    let mut expected = vec![Entity::Object(bears), Entity::Object(forest)];
    expected.sort();
    assert_eq!(offered, expected);
    assert!(t.in_hand(P0, "Forest"));
    // The other three are the bottom three cards, and the fifth card is on top.
    let lib = t.g.player(P0).library.clone();
    let mut bottom3: Vec<String> = lib[..3].iter().map(|c| name_of(&t, *c)).collect();
    bottom3.sort();
    let mut others: Vec<String> = [duress, bears, bolt].map(|c| name_of(&t, c)).to_vec();
    others.sort();
    assert_eq!(bottom3, others);
    assert_eq!(library_top_first(&t)[0], shock);
    // No order was asked: it's random.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::Order { .. })));
}

#[test]
fn scout_the_borders_may_take_nothing_and_mills_the_rest() {
    cr!("608.2d");
    assert_supported("Scout the Borders");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    stack(
        &mut t,
        &["Shock", "Grizzly Bears", "Lightning Bolt", "Forest", "Duress"],
    );
    let scout = t.hand(P0, "Scout the Borders");
    t.answer_choose(P0, &[]);
    let hand = t.hand_size(P0);
    t.cast(P0, scout).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.graveyard_size(P0), 6, "five cards and the spell");
}

#[test]
fn bond_of_flourishing_only_permanent_cards_and_life_either_way() {
    cr!("110.4a", "401.4");
    ruling!(
        "Bond of Flourishing",
        "A permanent card is an artifact, battle, creature, enchantment, land, or planeswalker card."
    );
    ruling!(
        "Bond of Flourishing",
        "You gain 3 life even if you don't reveal a permanent card"
    );
    assert_supported("Bond of Flourishing");
    for take in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        // Top first: Lightning Bolt, Forest, Grizzly Bears.
        let ids = stack(&mut t, &["Grizzly Bears", "Forest", "Lightning Bolt"]);
        let (bears, forest, bolt) = (ids[0], ids[1], ids[2]);
        let bond = t.hand(P0, "Bond of Flourishing");
        let chosen: Vec<Entity> = if take {
            vec![Entity::Object(bears)]
        } else {
            vec![]
        };
        t.answer_choose(P0, &chosen);
        t.cast(P0, bond).go();
        t.resolve();
        // The instant isn't a permanent card; the land and the creature are.
        let mut offered = last_choice_candidates(&t);
        offered.sort();
        let mut expected = vec![Entity::Object(bears), Entity::Object(forest)];
        expected.sort();
        assert_eq!(offered, expected);
        assert!(!offered.contains(&Entity::Object(bolt)));
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), take);
        assert_eq!(t.life(P0), 23);
        // The rest go to the bottom together, in the order P0 chooses.
        let rest = if take { 2 } else { 3 };
        assert!(t.asked().iter().any(|(p, d)| *p == P0
            && matches!(d, Decision::Order { items, .. } if items.len() == rest)));
        let lib = &t.g.player(P0).library;
        assert!(lib[..rest].iter().any(|c| name_of(&t, *c) == "Lightning Bolt"));
    }
}
