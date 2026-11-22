//! Rulings batch S35 — venturing into Undercity (CR 701.49d, 726.2): taking the
//! initiative (or having it as your upkeep begins) makes you venture into Undercity,
//! which moves you on in the dungeon you're in, or starts Undercity, and nothing else
//! does.

use crate::r_s01_common::supported;
use crate::r_s05_common::enter;
use crate::r_s06_common::activate_containing;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::dungeons;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const TOMB: &str = "Tomb of Annihilation";

/// The dungeon `p` is in and the room of their venture marker.
fn at(t: &TestGame, p: PlayerId) -> Option<(String, usize)> {
    dungeons::marker(&t.g, p).map(|(d, r)| (t.obj_now(d).chars.name.to_string(), r))
}

/// P0 owns these dungeon cards outside the game.
fn owning(names: &[&str]) -> TestGame {
    let mut t = TestGame::new(2);
    for n in names {
        t.custom(P0, (*card(n)).clone(), Zone::Outside(P0));
    }
    t
}

/// P0 discards a Radiant Solar ("{W}, Discard this card: Venture into the dungeon and you
/// gain 3 life."), choosing option `choice` if asked, and the abilities resolve.
fn solar_venture(t: &mut TestGame, choice: Option<usize>) {
    t.lands(P0, "Plains", 1);
    let solar = t.hand(P0, "Radiant Solar");
    if let Some(c) = choice {
        t.answer(P0, DecisionKind::Option, Answer::Index(c));
    }
    activate_containing(t, P0, solar, "Venture").expect("Radiant Solar");
    t.resolve_all();
}

/// The dungeon names P0 was offered since decision `from`.
fn dungeons_offered(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. }
                if *p == P0 && options.iter().any(|o| o.contains("Mine") || o == TOMB) =>
            {
                Some(options.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn venturing_into_undercity_starts_only_undercity() {
    cr!("701.49d", "726.2");
    ruling!(
        "Aarakocra Sneak",
        "Similarly, when instructed to venture into Undercity, you can't start a dungeon that isn't Undercity."
    );
    supported("Aarakocra Sneak");
    // "When this creature enters, you take the initiative." P0 owns other dungeons, but
    // venturing into Undercity from outside a dungeon starts Undercity, with no choice.
    let mut t = owning(&[TOMB, "Lost Mine of Phandelver"]);
    let from = t.asked().len();
    enter(&mut t, P0, "Aarakocra Sneak");
    t.resolve_all();
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    assert!(dungeons_offered(&t, from).is_empty());
}

#[test]
fn venturing_into_undercity_in_another_dungeon_moves_on_in_it() {
    cr!("701.49d", "726.2", "309.7");
    ruling!(
        "Goliath Paladin",
        "If you're already in a dungeon when instructed to venture into Undercity, you move to the next room of that dungeon. If you are already in the last room, you will complete that dungeon and start Undercity. This is true whether you're already in Undercity or any other dungeon."
    );
    supported("Goliath Paladin");
    supported("Aarakocra Sneak");
    supported("Trailblazer's Torch");
    // In Tomb of Annihilation (Trapped Entry), via Radiant Solar.
    let mut t = TestGame::new(2);
    solar_venture(&mut t, Some(2));
    assert_eq!(at(&t, P0), Some((TOMB.to_string(), 0)));
    // Goliath Paladin: "When this creature enters, you take the initiative." P0 moves on
    // in the Tomb (to Oubliette, the second of the two rooms it leads to).
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    enter(&mut t, P0, "Goliath Paladin");
    t.resolve_all();
    assert_eq!(at(&t, P0), Some((TOMB.to_string(), 3)));
    // Taking it again (Aarakocra Sneak): the last room, Cradle of the Death God.
    enter(&mut t, P0, "Aarakocra Sneak");
    while at(&t, P0) != Some((TOMB.to_string(), 4)) {
        assert!(t.stack_len() > 0, "{:?}", at(&t, P0));
        t.resolve();
    }
    // Once more (Trailblazer's Torch) while P0 is in that last room (its room ability is
    // still on the stack): the Tomb is completed, and Undercity starts.
    assert!(t.stack_len() > 0);
    enter(&mut t, P0, "Trailblazer's Torch");
    while t.g.player(P0).dungeons_completed == 0 {
        assert!(t.stack_len() > 0, "{:?}", at(&t, P0));
        t.resolve();
    }
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    t.resolve_all();
    // In Undercity, taking the initiative again moves on in Undercity.
    enter(&mut t, P0, "Goliath Paladin");
    t.resolve_all();
    let (name, room) = at(&t, P0).unwrap();
    assert_eq!(name, "Undercity");
    assert!(room > 0);
}

#[test]
fn only_the_initiative_ventures_into_undercity() {
    cr!("701.49a", "701.49d", "726.2");
    ruling!(
        "Sarevok's Tome",
        "You cannot venture into Undercity unless instructed to do so, either because you have the initiative at the beginning of your upkeep or because you take the initiative. Notably, if you aren't in a dungeon and an effect instructs you to venture into the dungeon (not venture into Undercity), you can't start Undercity."
    );
    supported("Sarevok's Tome");
    // Venturing into the dungeon (Radiant Solar) while in no dungeon: Undercity isn't
    // among the dungeons offered, though P0 owns it.
    let mut t = owning(&["Undercity"]);
    let from = t.asked().len();
    solar_venture(&mut t, None);
    let offered = dungeons_offered(&t, from);
    assert_eq!(offered.len(), 1);
    assert!(!offered[0].iter().any(|o| o == "Undercity"));
    assert_ne!(at(&t, P0).unwrap().0, "Undercity");
    // Sarevok's Tome: "When this artifact enters, you take the initiative." Then at the
    // beginning of P0's upkeep, P0 has the initiative: venture into Undercity.
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Sarevok's Tome");
    t.resolve_all();
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let (name, room) = at(&t, P0).unwrap();
    assert_eq!(name, "Undercity");
    assert!(room > 0);
}
