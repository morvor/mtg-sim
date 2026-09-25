//! CR 702.45 Bushido.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn bushido_triggers_when_blocking() {
    cr!("702.45a");
    let mut t = TestGame::new(2);
    // Devoted Retainer: 1/1, Bushido 1.
    let retainer = t.battlefield(P1, "Devoted Retainer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(retainer, bears)]);
    // 2/2 retainer and 2/2 bears trade.
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(retainer));
}

#[test]
fn prowess_pumps_on_noncreature_spell() {
    cr!("702.108a");
    let mut t = TestGame::new(2);
    // Monastery Swiftspear: 1/2 haste, prowess.
    let swift = t.battlefield(P0, "Monastery Swiftspear");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(swift), (2, 3));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(swift), (1, 2));
}
