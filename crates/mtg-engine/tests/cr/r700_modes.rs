//! CR 700.2d, 700.2e, 700.2h: choosing the same mode more than once, modes chosen by
//! another player, and modes with additional costs (spree).

use crate::r700_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_mode_chosen_several_times_is_performed_that_many_times() {
    cr!("700.2d");
    supported("Fiery Confluence");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Ornithopter");
    let conf = t.hand(P0, "Fiery Confluence");
    // "~ deals 2 damage to each opponent" twice, and "Destroy target artifact" once.
    let s = t.cast(P0, conf).modes(&[1, 2, 1]).target(a).go();
    assert_eq!(modes_of(&t, s), vec![1, 1, 2]);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert!(!t.g.is_live(a));
    assert!(t.g.is_live(b));
}

#[test]
fn a_repeated_mode_may_target_the_same_object_each_time() {
    cr!("700.2d");
    supported("Mystic Confluence");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let conf = t.hand(P0, "Mystic Confluence");
    // "Return target creature to its owner's hand" three times: the same creature twice
    // and another once.
    let s = t
        .cast(P0, conf)
        .modes(&[1, 1, 1])
        .target(bears)
        .target(bears)
        .target(giant)
        .go();
    assert_eq!(
        targets_of(&t, s),
        vec![
            Entity::Object(bears),
            Entity::Object(bears),
            Entity::Object(giant)
        ]
    );
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn normally_the_same_mode_cant_be_chosen_twice() {
    cr!("700.2d");
    supported("Cryptic Command");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let cmd = t.hand(P0, "Cryptic Command");
    // "Draw a card" twice isn't a legal choice for "Choose two —".
    let s = t.cast(P0, cmd).modes(&[3, 3]).go();
    let modes = modes_of(&t, s);
    assert_eq!(modes.len(), 2);
    assert_ne!(modes[0], modes[1]);
    let (min, max, repeat) = t
        .asked()
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes {
                min,
                max,
                allow_repeat,
                ..
            } => Some((*min, *max, *allow_repeat)),
            _ => None,
        })
        .unwrap();
    assert_eq!((min, max, repeat), (2, 2, false));
}

#[test]
fn an_opponent_chooses_the_mode() {
    cr!("700.2e");
    supported("Library of Lat-Nam");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let lib = t.hand(P0, "Library of Lat-Nam");
    let wanted = t.library_top(P0, "Black Lotus");
    // The caster would pick the search; the opponent picks "draw three at the next
    // upkeep".
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
    let s = t.cast(P0, lib).go();
    assert_eq!(modes_of(&t, s), vec![0]);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseModes { .. })));
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { .. })));
    t.resolve();
    assert!(t.g.is_live(wanted));
    assert_eq!(t.zone(wanted), mtg_engine::object::Zone::Library(P0));
}

#[test]
fn the_controller_decides_which_opponent_chooses_the_mode() {
    cr!("700.2e");
    supported("Library of Lat-Nam");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 5);
    let lib = t.hand(P0, "Library of Lat-Nam");
    t.library_top(P0, "Black Lotus");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P2, DecisionKind::Modes, Answer::Indices(vec![1]));
    let hand = t.hand_size(P0);
    let s = t.cast(P0, lib).go();
    assert_eq!(modes_of(&t, s), vec![1]);
    t.answer_choose(P0, &[]);
    t.resolve();
    // P2 chose the search mode: a card was put into P0's hand.
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_hand(P0, "Black Lotus"));
}

#[test]
fn each_chosen_mode_adds_its_additional_cost() {
    cr!("700.2h");
    supported("Unfortunate Accident");
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let acc = t.hand(P0, "Unfortunate Accident");
    // {B} + {2}{B} (destroy) + {1} (Mercenary token): all five Swamps.
    t.cast(P0, acc).modes(&[0, 1]).target(bears).go();
    assert!(swamps.iter().all(|s| t.obj(*s).tapped));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Mercenary Token").len(), 1);
    // Choosing only the token mode costs {B} + {1}.
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 5);
    let acc = t.hand(P0, "Unfortunate Accident");
    t.cast(P0, acc).modes(&[1]).go();
    assert_eq!(swamps.iter().filter(|s| t.obj(**s).tapped).count(), 2);
    // Without enough mana for both, both can't be chosen.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let acc = t.hand(P0, "Unfortunate Accident");
    assert!(t.cast(P0, acc).modes(&[0, 1]).target(bears).try_go().is_err());
}
