//! Immersturm (compiled in rulings batch P089, with Pandemonium): "Whenever a creature
//! enters, that creature's controller may have it deal damage equal to its power to any
//! target of their choice." The planar controller controls the triggered ability
//! (CR 901.6, 603.3a); the entering creature's controller chooses the target and whether
//! it deals damage. "Whenever chaos ensues, exile target creature, then return it to the
//! battlefield under its owner's control."

use crate::r703_common::supported;
use crate::r900_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn immersturm_entering_creatures_controller_chooses_the_target() {
    cr!("901.6", "603.3a", "603.3d");
    supported("Immersturm");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Immersturm"]);
    planechase::set_starting_plane(&mut t.g);
    let from = t.asked().len();
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_yes(P1, true);
    let giant = t.enter(P1, "Hill Giant");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P0, the planar controller, controls the ability; P1 chose its target.
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(trig).controller, P0);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.on_battlefield(giant));
}

#[test]
fn immersturm_chaos_flickers_target_creature() {
    cr!("311.7", "400.7");
    supported("Immersturm");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Immersturm"]);
    planechase::set_starting_plane(&mut t.g);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    // The Bears' return triggers the first ability: P1 declines.
    t.answer_yes(P1, false);
    roll_effect(&mut t, P0, PlanarFace::Chaos);
    t.resolve_all();
    let back = t.g.current(bears);
    assert_ne!(back, bears);
    assert!(t.on_battlefield(back));
    assert_eq!(t.obj_now(back).controller, P1);
    assert_eq!(t.counters(back, "+1/+1"), 0);
    assert_eq!(t.life(P0), 20);
}
