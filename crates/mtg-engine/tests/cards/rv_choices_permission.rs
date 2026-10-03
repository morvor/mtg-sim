//! Permissions for a target card the same text moved first ("exile target ... card from
//! a graveyard. You may cast it this turn"): the permission follows the card to its new
//! zone (CR 400.7, 610.3).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn dire_fleet_daredevil_exiled_card_can_be_cast() {
    cr!("400.7");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let shock = t.graveyard(P1, "Shock");
    t.answer_targets(P0, &[Entity::Object(shock)]);
    t.enter(P0, "Dire Fleet Daredevil");
    t.settle();
    t.resolve();
    let exiled = t.g.current(shock);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, exiled).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
}
