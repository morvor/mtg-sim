//! Structure-coverage tests (round 4): keyword structures shared by many cards that no
//! other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::act;
use super::structure_seed_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hybrid_cycling_draws_a_card() {
    cr!("702.29a", "107.4e", "701.9a");
    // Monstrous Carabid: "Cycling {B/R}"
    supported("Monstrous Carabid");
    for land in ["Swamp", "Mountain"] {
        let mut t = TestGame::new(2);
        let c = t.hand(P0, "Monstrous Carabid");
        let on_bf = t.battlefield(P0, "Monstrous Carabid");
        assert!(act(&mut t, P0, c, "cycling", &[]).is_err(), "no mana");
        t.battlefield(P0, "Island");
        assert!(
            act(&mut t, P0, c, "cycling", &[]).is_err(),
            "{{U}} can't pay {{B/R}}"
        );
        let l = t.battlefield(P0, land);
        assert!(
            act(&mut t, P0, on_bf, "cycling", &[]).is_err(),
            "only from hand"
        );
        let lib = t.library_size(P0);
        act(&mut t, P0, c, "cycling", &[]).unwrap();
        assert!(t.obj_now(l).tapped, "{land} paid {{B/R}}");
        assert!(t.in_graveyard(P0, "Monstrous Carabid"), "discarded");
        t.resolve_all();
        assert_eq!(t.library_size(P0), lib - 1);
        assert_eq!(t.hand_size(P0), 1);
    }
}
