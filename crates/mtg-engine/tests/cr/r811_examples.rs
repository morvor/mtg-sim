//! The worked example of CR 811 (the Alternating Teams variant), played out as a test.

use mtg_engine::game::GameConfig;
use mtg_engine::multiplayer::setup;
use mtg_engine::*;

#[test]
fn three_teams_are_seated_a1_b1_c1_a2_b2_c2_and_so_on() {
    cr!("811.3");
    cr_example!("811.3", "A1, B1, C1, A2, B2, C2, A3, B3, C3");
    // Participants 0–2 are team A, 3–5 team B, 6–8 team C.
    let g = setup::new_seated(
        GameConfig::alternating_teams(vec![0, 0, 0, 1, 1, 1, 2, 2, 2]),
        (0..9).map(|_| super::r100_common::fillers(5)).collect(),
        vec![],
    );
    assert_eq!(g.multiplayer.seats, vec![0, 3, 6, 1, 4, 7, 2, 5, 8]);
    assert_eq!(g.config.teams, Some(vec![0, 1, 2, 0, 1, 2, 0, 1, 2]));
    assert_eq!(
        GameConfig::alternating_teams(vec![0, 1, 2, 0, 1, 2, 0, 1, 2]).validate(9),
        Ok(())
    );
}
