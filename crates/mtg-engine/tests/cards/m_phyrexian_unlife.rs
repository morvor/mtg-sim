//! Phyrexian Unlife (hand-written, `src/cards/phyrexian_unlife.rs`): at 0 or less life,
//! damage is dealt to its controller as though its source had infect (CR 120.3b).

use mtg_engine::testing::*;
use mtg_engine::*;

fn bolt_p0(t: &mut TestGame) {
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
}

#[test]
fn damage_from_positive_life_is_dealt_normally_then_as_infect() {
    cr!("120.3a", "120.3b", "104.3b");
    ruling!("Phyrexian Unlife", "won't affect damage that reduces your life total from a positive number to 0 or less");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Unlife");
    t.g.players[0].life = 1;
    bolt_p0(&mut t);
    assert_eq!(t.life(P0), -2);
    assert!(!t.has_lost(P0));
    assert_eq!(t.g.player(P0).poison(), 0);
    bolt_p0(&mut t);
    assert_eq!(t.life(P0), -2);
    assert_eq!(t.g.player(P0).poison(), 3);
}

#[test]
fn only_its_controller_is_affected() {
    cr!("120.3a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Unlife");
    t.g.players[0].life = -1;
    t.g.players[1].life = 2;
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.g.player(P1).poison(), 0);
    assert_eq!(t.life(P1), -1);
}
