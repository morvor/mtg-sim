//! Granted abilities that name the granting object (CR 113.7, 201.5a): Nettlevine Blight.

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn nettlevine_blight_moves_on_after_its_host_is_sacrificed() {
    cr!("113.7", "201.5a", "701.3a");
    supported("Nettlevine Blight");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Hill Giant");
    let blight = t.battlefield(P0, "Nettlevine Blight");
    t.g.attach(blight, Entity::Object(bears));
    // P0's own end step: the ability belongs to P1's permanent ("your end step").
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // P1's end step: P1 sacrifices the bears and attaches the Blight to a creature or
    // land P1 controls.
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let b = t.g.obj(t.g.current(blight));
    assert_eq!(b.attached_to, Some(Entity::Object(forest)));
    assert_eq!(b.controller, P0);
    assert!(t.on_battlefield(mine));
}
