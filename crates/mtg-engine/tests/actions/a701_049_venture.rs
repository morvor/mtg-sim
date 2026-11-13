//! CR 701.49: venture into the dungeon.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{KeywordAction, Sel};
use mtg_engine::dungeons;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const MINE: &str = "Lost Mine of Phandelver";
const TOMB: &str = "Tomb of Annihilation";

fn venture(t: &mut TestGame) {
    run(t, P0, None, ka(KeywordAction::Venture, Sel::None, 1), &[]);
}

fn venture_into_undercity(t: &mut TestGame) {
    run(
        t,
        P0,
        None,
        mtg_engine::kwa::venture::venture_into("Undercity"),
        &[],
    );
}

/// The dungeon in P0's command zone and the room of the venture marker.
fn at(t: &TestGame) -> Option<(String, usize)> {
    dungeons::marker(&t.g, P0).map(|(d, r)| (t.obj_now(d).chars.name.to_string(), r))
}

/// P0 owns these dungeon cards outside the game.
fn owning(names: &[&str]) -> TestGame {
    let mut t = TestGame::new(2);
    for n in names {
        t.custom(P0, (*card::card(n)).clone(), Zone::Outside(P0));
    }
    t
}

#[test]
fn without_a_dungeon_the_player_chooses_one_they_own_and_enters_its_topmost_room() {
    cr!("701.49a");
    ruling!(
        "Zombie Ogre",
        "The player venturing into the dungeon chooses which dungeon they will venture into. They may choose a dungeon that they have already completed this game."
    );
    let mut t = owning(&[MINE, TOMB]);
    option(&mut t, P0, 1);
    venture(&mut t);
    assert_eq!(at(&t), Some((TOMB.to_string(), 0)));
    // Its topmost room's ability: "Trapped Entry — Each player loses 1 life."
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19);
    // Complete it (Trapped Entry → Oubliette → Cradle of the Death God), then venture
    // again: the completed Tomb may be chosen again.
    option(&mut t, P0, 1);
    venture(&mut t);
    t.resolve_all();
    venture(&mut t);
    t.resolve_all();
    assert_eq!(at(&t), None);
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    option(&mut t, P0, 1);
    venture(&mut t);
    assert_eq!(at(&t), Some((TOMB.to_string(), 0)));
}

#[test]
fn the_player_moves_their_marker_along_an_arrow_of_their_choice() {
    cr!("701.49b");
    ruling!(
        "Zombie Ogre",
        "You can only move forward (well, downward) in a dungeon, never backwards or sideways."
    );
    let mut t = owning(&[MINE]);
    venture(&mut t);
    t.resolve_all();
    // Cave Entrance leads to Goblin Lair (room 1) or Mine Tunnels (room 2).
    option(&mut t, P0, 1);
    venture(&mut t);
    assert_eq!(at(&t), Some((MINE.to_string(), 2)));
    // Only the rooms the arrows point to were offered.
    let offered: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 2);
    assert!(offered[0][0].starts_with("Goblin Lair"));
    assert!(offered[0][1].starts_with("Mine Tunnels"));
    // Mine Tunnels leads to Dark Pool (4) or Fungi Cavern (5); a room with one arrow
    // doesn't ask.
    t.resolve_all();
    option(&mut t, P0, 0);
    venture(&mut t);
    assert_eq!(at(&t), Some((MINE.to_string(), 4)));
    t.resolve_all();
    let asked = t.asked().len();
    venture(&mut t);
    assert_eq!(at(&t), Some((MINE.to_string(), 6)));
    assert_eq!(t.asked().len(), asked);
}

#[test]
fn from_the_bottommost_room_the_player_completes_the_dungeon_and_starts_another() {
    cr!("701.49c");
    ruling!(
        "Zombie Ogre",
        "If you somehow venture into the dungeon while a room's ability is on the stack, you will continue on in the dungeon. If you're already in the last room, complete that dungeon and start a new one."
    );
    let mut t = owning(&[MINE, TOMB]);
    // Tomb of Annihilation: Trapped Entry → Oubliette → Cradle of the Death God.
    option(&mut t, P0, 1);
    venture(&mut t);
    option(&mut t, P0, 1);
    venture(&mut t);
    venture(&mut t);
    assert_eq!(at(&t), Some((TOMB.to_string(), 4)));
    // Cradle of the Death God's ability is on the stack: the dungeon isn't completed yet.
    t.settle();
    assert_eq!(t.g.player(P0).dungeons_completed, 0);
    // Venturing now completes it, then P0 chooses a new dungeon.
    option(&mut t, P0, 0);
    venture(&mut t);
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    assert_eq!(at(&t), Some((MINE.to_string(), 0)));
}

#[test]
fn venture_into_undercity_starts_only_undercity() {
    cr!("701.49d");
    ruling!(
        "Explore the Underdark",
        "Similarly, when instructed to venture into Undercity, you can’t start a dungeon that isn’t Undercity."
    );
    ruling!(
        "Explore the Underdark",
        "If you aren’t in a dungeon when instructed to venture into Undercity, you will put Undercity into the command zone and move your venture marker to Secret Entrance (the first room)."
    );
    // Not in a dungeon: Undercity, although P0 owns other dungeons.
    let mut t = owning(&[MINE, TOMB]);
    venture_into_undercity(&mut t);
    assert_eq!(at(&t), Some(("Undercity".to_string(), 0)));
    // Secret Entrance: "Search your library for a basic land card, reveal it, put it into
    // your hand, then shuffle."
    t.library_top(P0, "Forest");
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn venture_into_undercity_while_in_a_dungeon_follows_the_normal_procedure() {
    cr!("701.49d");
    ruling!(
        "Explore the Underdark",
        "If you’re already in a dungeon when instructed to venture into Undercity, you move to the next room of that dungeon. If you are already in the last room, you will complete that dungeon and start Undercity."
    );
    let mut t = owning(&[TOMB]);
    venture(&mut t);
    option(&mut t, P0, 1);
    // Trapped Entry leads to Veils of Fear (room 1) or Oubliette (room 3).
    venture_into_undercity(&mut t);
    assert_eq!(at(&t), Some((TOMB.to_string(), 3)));
    venture_into_undercity(&mut t);
    assert_eq!(at(&t), Some((TOMB.to_string(), 4)));
    t.settle();
    // From the last room: complete it and start Undercity.
    venture_into_undercity(&mut t);
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    assert_eq!(at(&t), Some(("Undercity".to_string(), 0)));
}

#[test]
fn venturing_into_the_dungeon_never_starts_undercity() {
    cr!("701.49d");
    ruling!(
        "Explore the Underdark",
        "if you aren’t in a dungeon and an effect instructs you to venture into the dungeon (not venture into Undercity), you can’t start Undercity."
    );
    let mut t = owning(&["Undercity", MINE]);
    venture(&mut t);
    assert_eq!(at(&t), Some((MINE.to_string(), 0)));
    // With only Undercity, the standard dungeons are used.
    let mut t = owning(&["Undercity"]);
    venture(&mut t);
    let (name, room) = at(&t).expect("in a dungeon");
    assert_ne!(name, "Undercity");
    assert_eq!(room, 0);
    let offered: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![vec![
            "Lost Mine of Phandelver".to_string(),
            "Dungeon of the Mad Mage".to_string(),
            "Tomb of Annihilation".to_string()
        ]]
    );
}
