//! CR 506.8, 506.8d: a spell that may be cast "only after combat" can't be cast before
//! the turn's combat phase and can be cast once it has ended.

use crate::r506_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn glyph_of_reincarnation_can_be_cast_only_after_combat() {
    cr!("506.8", "506.8d");
    // "Cast this spell only after combat. Destroy all creatures that were blocked by
    // target Wall this turn. ..." (an instant).
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let wall = t.battlefield(P1, "Wall of Stone");
    let glyph = t.hand(P0, "Glyph of Reincarnation");
    assert!(t.cast(P0, glyph).target(Entity::Object(wall)).try_go().is_err());
    t.clear_answers();
    declare(&mut t, &[]);
    go_to(&mut t, Step::PostcombatMain);
    t.cast(P0, glyph).target(Entity::Object(wall)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Glyph of Reincarnation"));
    assert!(t.on_battlefield(wall));
}
