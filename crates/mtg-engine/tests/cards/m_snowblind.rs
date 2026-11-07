//! Snowblind (hand-written, `src/cards/snowblind.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn shrinks_by_its_controllers_snow_lands_keeping_toughness_at_least_1() {
    cr!("613.4c");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Snow-Covered Forest", 2);
    let aura = t.battlefield(P0, "Snowblind");
    t.g.attach(aura, Entity::Object(giant));
    t.g.recompute();
    // X = 2; Y = min(2, 3 - 1) = 2.
    assert_eq!(t.pt(giant), (1, 1));
    t.lands(P1, "Snow-Covered Forest", 2);
    t.g.recompute();
    // X = 4; Y = min(4, 2) = 2.
    assert_eq!(t.pt(giant), (-1, 1));
}

#[test]
fn an_attacker_counts_the_defending_players_snow_lands() {
    cr!("613.4c", "508.1");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Snow-Covered Island", 1);
    let aura = t.battlefield(P1, "Snowblind");
    t.g.attach(aura, Entity::Object(giant));
    t.g.recompute();
    // Not attacking: P0 has no snow lands.
    assert_eq!(t.pt(giant), (3, 3));
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    let _ = Step::End;
}
