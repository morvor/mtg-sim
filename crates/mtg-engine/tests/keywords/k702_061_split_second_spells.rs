//! CR 702.61 Split second on spells ("Instant and sorcery spells you control have split
//! second").

use crate::common_k702_011_017::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn instant_and_sorcery_spells_you_control_have_split_second() {
    cr!("702.61a");
    assert_supported("Samut, Tyrant of Naktamun");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Samut, Tyrant of Naktamun");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    t.lands(P1, "Island", 2);
    let counterspell = t.hand(P1, "Counterspell");
    // While P0's instant is on the stack, other spells can't be cast and nonmana
    // abilities can't be activated.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.cast(P1, counterspell).try_go().is_err());
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_err());
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Once it has left the stack, they can again.
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_ok());
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // An opponent's instant doesn't have split second: P0 can respond to it.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(P0).go();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_ok());
}
