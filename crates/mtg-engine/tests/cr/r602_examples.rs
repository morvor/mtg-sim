//! The worked examples of CR 602 (activating activated abilities), played out as tests.

use crate::r600_examples_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn two_generic_and_tap_is_two_mana_of_any_type_plus_tapping_the_permanent() {
    cr!("602.1a", "602.2b");
    cr_example!("602.1a", "{2}, {T}: You gain 1 life");
    let mut t = TestGame::new(2);
    let def = custom_card("Life Stone", "Artifact", None, "{2}, {T}: You gain 1 life.");
    let stone = t.custom(P0, def, Zone::Battlefield);
    // Two mana of any type: a green and a blue.
    let forest = t.battlefield(P0, "Forest");
    let island = t.battlefield(P0, "Island");
    t.activate(P0, stone, 0, &[]).unwrap();
    assert!(t.obj(forest).tapped && t.obj(island).tapped);
    assert!(t.obj(stone).tapped);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    // Tapping the permanent is part of the cost: it can't be activated while tapped.
    t.lands(P0, "Swamp", 2);
    assert!(t.activate(P0, stone, 0, &[]).is_err());
    t.g.untap(stone);
    t.activate(P0, stone, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 22);
}
