//! "You may play lands from your graveyard." (Crucible of Worlds): the land play still
//! follows the normal limits and timing (CR 305.1–305.3).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn crucible_of_worlds_plays_a_land_from_the_graveyard() {
    cr!("305.1", "305.2");
    ruling!(
        "Crucible of Worlds",
        "You can still play only one land per turn, and only during your main phase"
    );
    assert_supported("Crucible of Worlds");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    let island = t.graveyard(P0, "Island");
    // Without the permission, a land in the graveyard can't be played.
    assert!(t.play_land(P0, forest).is_err());
    t.battlefield(P0, "Crucible of Worlds");
    t.play_land(P0, forest).unwrap();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // Still only one land per turn.
    assert!(t.play_land(P0, island).is_err());
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn crucible_of_worlds_only_on_your_turn_in_a_main_phase() {
    cr!("305.1", "305.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crucible of Worlds");
    let forest = t.graveyard(P0, "Forest");
    // Not during combat...
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.play_land(P0, forest).is_err());
    // ...and not on another player's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.play_land(P0, forest).is_err());
    // An opponent can't use your Crucible for their graveyard.
    let mountain = t.graveyard(P1, "Mountain");
    assert!(t.play_land(P1, mountain).is_err());
    assert!(t.in_graveyard(P0, "Forest"));
}
