//! Regression: paying mana with lands that have several basic land types.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn repro_bolt_with_volcanic_island() {
    cr!("305.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Volcanic Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    let r = t.cast(P0, bolt).target(Entity::Player(P1)).try_go();
    assert!(r.is_ok(), "{r:?}\n{}", t.dump_log());
    t.resolve();
    assert_eq!(t.life(P1), 17);
}
