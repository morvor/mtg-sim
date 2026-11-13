//! Rulings batch P062 — "it doesn't untap during its controller's next untap step" tracks
//! the creature, not its controller: if it changes controllers before its first
//! controller's next untap step, it doesn't untap during its new controller's next untap
//! step (CR 502.3, 611.2a). And "tap all nonblue creatures" affects the creatures of
//! every player.

use crate::r_p062_common::*;
use crate::r_s01_common::supported;
use crate::r_s05_common::enter;
use crate::r_s06_common::give_control;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::testing::*;
use mtg_engine::*;

/// After P0's spell or ability made P1's tapped `creature` not untap during its
/// controller's next untap step, P0 gains control of it: it doesn't untap during P1's next
/// untap step (P0 controls it), nor during P0's next one; it untaps in P0's untap step
/// after that.
fn p0_steals_it(t: &mut TestGame, creature: ObjectId) {
    give_control(t, creature, P0);
    through_untap_step(t, P1);
    assert!(is_tapped(t, creature));
    misses_one_untap(t, creature, P0);
}

/// P0 casts the real spell `name` targeting P1's tapped Bears, then steals them.
fn spell_then_steal(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    cast_and_resolve(&mut t, P0, name, &[obj(bears)]);
    p0_steals_it(&mut t, bears);
}

#[test]
fn grip_of_the_roil_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Grip of the Roil",
        "Grip of the Roil tracks the creature, but not its controller. If the creature changes controllers before its first controller's next untap step has come around, then it won't untap during its new controller's next untap step."
    );
    spell_then_steal("Grip of the Roil");
}

#[test]
fn rush_of_ice_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Rush of Ice",
        "Rush of Ice tracks the creature, but not its controller. If the creature changes controllers before its first controller’s next untap step has come around, then it won’t untap during its new controller’s next untap step."
    );
    spell_then_steal("Rush of Ice");
}

#[test]
fn icy_blast_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Icy Blast",
        "Icy Blast tracks the creature, but not its controller. If the creature changes controllers before its first controller’s next untap step has come around, then it won’t untap during its new controller’s next untap step."
    );
    supported("Icy Blast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let blast = t.hand(P0, "Icy Blast");
    t.cast(P0, blast).x(1).target(bears).go();
    t.resolve_all();
    p0_steals_it(&mut t, bears);
}

#[test]
fn skyline_cascade_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Skyline Cascade",
        "The triggered ability tracks the creature, but not its controller. If the creature changes controllers before its first controller's next untap step has come around, then it won't untap during its new controller's next untap step."
    );
    supported("Skyline Cascade");
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Skyline Cascade");
    t.resolve_all();
    p0_steals_it(&mut t, bears);
}

/// P0 casts Breaching Leviathan from hand (P0 controls `blue` and `mine`, P1 `theirs`).
fn leviathan(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    supported("Breaching Leviathan");
    let blue = t.battlefield(P0, "Wind Drake");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_and_resolve(t, P0, "Breaching Leviathan", &[]);
    (blue, mine, theirs)
}

#[test]
fn breaching_leviathan_affects_all_nonblue_creatures_including_yours() {
    cr!("502.3", "701.26a");
    ruling!(
        "Breaching Leviathan",
        "The triggered ability affects all nonblue creatures, including those you control."
    );
    let mut t = TestGame::new(2);
    let (blue, mine, theirs) = leviathan(&mut t);
    let lev = t.named_on_battlefield("Breaching Leviathan")[0];
    assert!(!is_tapped(&t, blue));
    assert!(!is_tapped(&t, lev), "Breaching Leviathan is blue");
    assert!(is_tapped(&t, mine));
    assert!(is_tapped(&t, theirs));
    misses_one_untap(&mut t, theirs, P1);
    // (P0's untap steps passed too: P0's Bears missed the first one.)
    let mut t = TestGame::new(2);
    let (_, mine, _) = leviathan(&mut t);
    misses_one_untap(&mut t, mine, P0);
}

#[test]
fn breaching_leviathan_tracks_the_creatures_not_their_controllers() {
    cr!("502.3", "611.2a");
    ruling!(
        "Breaching Leviathan",
        "The triggered ability tracks the creatures, but not their controllers. If any of the creatures changes controllers before its original controller’s next untap step has come around, that creature won’t untap during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let (_, _, theirs) = leviathan(&mut t);
    p0_steals_it(&mut t, theirs);
}
