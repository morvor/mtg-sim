//! CR 400.7, 702.106a, 905.4a: a conspiracy put into the command zone from outside the
//! game is a new object; those with hidden agenda are face down, and their ids don't
//! follow the order of the cards outside the game, which would tell them apart (CR 406.3,
//! 400.2). Found by the hidden information property (`mtg_api::hidden`, docs/PROPERTIES.md).

use crate::r100_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn face_down_conspiracies_are_new_objects_numbered_in_no_particular_order() {
    cr!("400.7", "702.106a", "905.4a");
    let names = ["Brago's Favor", "Double Stroke", "Muzzio's Preparations"];
    let mut in_order = 0;
    for seed in 0..8 {
        let mut t = pregame(
            GameConfig {
                skip_mulligans: true,
                starting_player: Some(P0),
                seed,
                ..GameConfig::conspiracy_draft_game()
            },
            vec![fillers(40), fillers(40)],
        );
        let side =
            t.g.add_to_sideboard(P0, names.iter().map(|n| card(n)).collect());
        t.g.start();
        let now: Vec<ObjectId> = side.iter().map(|c| t.g.current(*c)).collect();
        for (old, new) in side.iter().zip(&now) {
            assert_ne!(old, new, "a new object (CR 400.7)");
            assert_eq!(t.g.obj(*new).zone, Zone::Command);
            assert!(t.g.obj(*new).face_down);
        }
        let mut sorted = now.clone();
        sorted.sort();
        in_order += usize::from(sorted == now);
    }
    // With three cards, the ids follow the sideboard's order in about one game in six.
    assert!(
        in_order < 8,
        "the ids always follow the cards' order outside the game"
    );
}
