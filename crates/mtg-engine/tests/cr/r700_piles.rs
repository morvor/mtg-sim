//! CR 700.3: separating objects into piles.

use crate::r114_common::{probe_lines, spy};
use crate::r700_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts five named cards on top of P0's library (the last one on top).
fn stack_library(t: &mut TestGame) -> Vec<ObjectId> {
    [
        "Grizzly Bears",
        "Hill Giant",
        "Lightning Bolt",
        "Island",
        "Ornithopter",
    ]
    .iter()
    .map(|n| t.library_top(P0, n))
    .collect()
}

#[test]
fn fact_or_fiction_separates_revealed_cards_into_two_piles() {
    cr!("700.3", "700.3a", "700.3b", "700.3c");
    supported("Fact or Fiction");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let cards = stack_library(&mut t);
    // While P1 separates the cards, they're still in P0's library.
    let watched = cards.clone();
    let log = spy(&mut t, P1, move |g, _p, d| match d {
        Decision::ChooseEntities { candidates, .. } => Some(format!(
            "{} {}",
            candidates.len(),
            watched
                .iter()
                .all(|c| g.is_live(*c) && matches!(g.obj(*c).zone, Zone::Library(_)))
        )),
        _ => None,
    });
    // P1 puts Hill Giant and Lightning Bolt in one pile; the rest form the other.
    t.answer_choose(P1, &[Entity::Object(cards[1]), Entity::Object(cards[2])]);
    // P0 takes the pile of three.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let fof = t.hand(P0, "Fact or Fiction");
    let hand = t.hand_size(P0);
    t.cast(P0, fof).go();
    t.resolve();
    assert_eq!(probe_lines(&log), vec!["5 true".to_string()]);
    // Each card went to exactly one place, one by one.
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Island"));
    assert!(t.in_hand(P0, "Ornithopter"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.graveyard_size(P0), 3); // and Fact or Fiction
}

#[test]
fn a_pile_can_be_empty() {
    cr!("700.3d");
    supported("Fact or Fiction");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let cards = stack_library(&mut t);
    // Everything in one pile.
    let all: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P1, &all);
    // P0 chooses the empty pile.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let fof = t.hand(P0, "Fact or Fiction");
    let hand = t.hand_size(P0);
    t.cast(P0, fof).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.graveyard_size(P0), 6);
}

#[test]
fn a_player_sacrifices_the_pile_of_their_choice() {
    cr!("700.3", "700.3a");
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    t.g.objects[lili.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Grizzly Bears");
    // P0 separates P1's permanents: the Bears alone, and the rest.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    // P1 chooses to sacrifice the pile with the Bears.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, lili, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(!t.g.is_live(bears));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(forest));
    assert!(t.on_battlefield(mine));
    // Only P1's permanents could be put into piles.
    let cands = t
        .asked()
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(cands.len(), 3);
    assert!(!cands.contains(&Entity::Object(mine)));
}

#[test]
fn separating_a_graveyard_into_piles_keeps_its_order() {
    cr!("700.3c");
    supported("Death or Glory");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Lightning Bolt");
    let c = t.graveyard(P0, "Hill Giant");
    let d = t.graveyard(P0, "Ornithopter");
    let before = t.g.player(P0).graveyard.clone();
    // When the opponent chooses a pile, the graveyard's order hasn't changed.
    let log = spy(&mut t, P1, move |g, _p, dec| match dec {
        Decision::ChooseOption { prompt, .. } if prompt == "Choose a pile" => {
            Some(format!("{:?}", g.player(P0).graveyard))
        }
        _ => None,
    });
    // P0 puts Hill Giant (the third card) alone in the first pile; the second pile is the
    // Bears and Ornithopter (Lightning Bolt isn't a creature card).
    t.answer_choose(P0, &[Entity::Object(c)]);
    // P1 exiles the Bears and Ornithopter.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let dog = t.hand(P0, "Death or Glory");
    t.cast(P0, dog).go();
    t.resolve();
    assert_eq!(probe_lines(&log), vec![format!("{before:?}")]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Ornithopter"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.g.is_live(b));
    let _ = (a, d);
}
