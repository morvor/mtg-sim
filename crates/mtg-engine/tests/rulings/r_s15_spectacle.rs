//! Rulings batch S15 — spectacle (CR 702.137): Light Up the Stage.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::untapped_lands;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const SPECTACLE: CastMethod = CastMethod::Keyword(KeywordKind::Spectacle);

#[test]
fn the_spectacle_cost_doesnt_depend_on_how_much_life_or_how_many_opponents_lost() {
    cr!("702.137a", "601.2f");
    ruling!(
        "Light Up the Stage",
        "A card's spectacle cost is the same no matter how much life your opponents lost or how many opponents lost life."
    );
    supported("Light Up the Stage");
    // Light Up the Stage: {2}{R} sorcery, spectacle {R}. Its opponents lose 1 life, 10
    // life, or (in a four-player game) each of three opponents loses 5 life: casting it
    // for its spectacle cost always costs {R}.
    for (players, losses) in [
        (2, vec![(P1, 1)]),
        (2, vec![(P1, 10)]),
        (4, vec![(P1, 5), (P2, 5), (P3, 5)]),
    ] {
        let mut t = TestGame::new(players);
        t.lands(P0, "Mountain", 3);
        let lus = t.hand(P0, "Light Up the Stage");
        for (p, n) in &losses {
            t.g.lose_life(*p, *n);
        }
        assert!(can_cast(&mut t, P0, lus, SPECTACLE));
        t.cast(P0, lus).method(SPECTACLE).go();
        assert_eq!(untapped_lands(&t, P0), 2, "{losses:?}");
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Light Up the Stage"));
    }
}
