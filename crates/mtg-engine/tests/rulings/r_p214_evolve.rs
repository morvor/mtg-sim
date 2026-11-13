//! Rulings batch P214 — evolve (CR 702.100): the comparison is made as the creature
//! enters and again as the ability resolves (an intervening "if" clause), power to power
//! and toughness to toughness.

use crate::r_p214_common::*;
use crate::r_s01_common::*;
use crate::r_s05_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn the_greater_stat_may_change_before_evolve_resolves_pollywog_prodigy() {
    cr!("702.100a", "603.4");
    ruling!(
        "Pollywog Prodigy",
        "When comparing the stats as the evolve ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on the creature with evolve."
    );
    supported("Pollywog Prodigy");
    // Pollywog Prodigy: 1/3, evolve. Hill Giant (3/3) has the greater power; in response
    // it gets -3/+1 (0/4): now its toughness is greater.
    let mut t = TestGame::new(2);
    let frog = t.battlefield(P0, "Pollywog Prodigy");
    let giant = enter(&mut t, P0, "Hill Giant");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    pump(&mut t, giant, -3, 1);
    assert_eq!(t.pt(giant), (0, 4));
    t.resolve_all();
    assert_eq!(t.counters(frog, counters::PLUS1), 1);
    assert_eq!(t.pt(frog), (2, 4));
}

#[test]
fn the_greater_stat_may_change_before_evolve_resolves_ray_fillet() {
    cr!("702.100a", "603.4");
    ruling!(
        "Ray Fillet, Wave Warrior",
        "When comparing the stats as the evolve ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on the creature with evolve."
    );
    supported("Ray Fillet, Wave Warrior");
    // Ray Fillet: 0/2. Memnite (1/1) has the greater power; in response it gets -1/+2
    // (0/3): now its toughness is greater.
    let mut t = TestGame::new(2);
    let ray = t.battlefield(P0, "Ray Fillet, Wave Warrior");
    let memnite = enter(&mut t, P0, "Memnite");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    pump(&mut t, memnite, -1, 2);
    assert_eq!(t.pt(memnite), (0, 3));
    t.resolve_all();
    assert_eq!(t.counters(ray, counters::PLUS1), 1);
}

#[test]
fn the_greater_stat_may_change_before_evolve_resolves_watchful_radstag() {
    cr!("702.100a", "603.4");
    ruling!(
        "Watchful Radstag",
        "When comparing the stats as the evolve ability resolves, it’s possible that the stat that’s greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you’ll put a +1/+1 counter on the creature with evolve. For example, if you control a 2/2 creature with evolve and a 1/3 creature enters the battlefield under your control, its toughness is greater"
    );
    supported("Watchful Radstag");
    // Watchful Radstag: 2/2, evolve. Wall of Wood (0/3) has the greater toughness; in
    // response it gets +3/-2 (3/1): now its power is greater.
    let mut t = TestGame::new(2);
    let stag = t.battlefield(P0, "Watchful Radstag");
    let wall = enter(&mut t, P0, "Wall of Wood");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    pump(&mut t, wall, 3, -2);
    assert_eq!(t.pt(wall), (3, 1));
    t.resolve_all();
    assert_eq!(t.counters(stag, counters::PLUS1), 1);
    assert_eq!(t.pt(stag), (3, 3));
}

#[test]
fn evolve_compares_power_to_power_and_toughness_to_toughness() {
    cr!("702.100a");
    ruling!(
        "Dinosaur Egg",
        "When comparing the stats of the two creature for evolve, you always compare power to power and toughness to toughness."
    );
    supported("Dinosaur Egg");
    // Dinosaur Egg: 0/3, evolve. Ornithopter (0/2): its toughness 2 is greater than the
    // Egg's power 0, but toughness is compared to toughness: no trigger.
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Dinosaur Egg");
    assert_eq!(t.pt(egg), (0, 3));
    enter(&mut t, P0, "Ornithopter");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
    // Grizzly Bears (2/2): its power 2 isn't greater than the Egg's toughness 3, but power
    // is compared to power: it triggers.
    enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(t.counters(egg, counters::PLUS1), 1);
}

#[test]
fn evolve_doesnt_trigger_unless_a_stat_is_greater_scurry_oak() {
    cr!("702.100a", "603.4");
    ruling!(
        "Scurry Oak",
        "Whenever a creature enters the battlefield under your control, check its power and toughness against the power and toughness of the creature with evolve. If neither characteristic of the new creature is greater, evolve won't trigger at all."
    );
    supported("Scurry Oak");
    // Scurry Oak: 1/2. Memnite (1/1): neither is greater. Grizzly Bears (2/2): power is.
    let mut t = TestGame::new(2);
    let oak = t.battlefield(P0, "Scurry Oak");
    enter(&mut t, P0, "Memnite");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
    enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(oak, counters::PLUS1), 1);
}

#[test]
fn evolve_doesnt_trigger_unless_a_stat_is_greater_gluttonous_slug() {
    cr!("702.100a", "603.4");
    ruling!(
        "Gluttonous Slug",
        "Whenever a creature enters the battlefield under your control, check its power and toughness against the power and toughness of the creature with evolve. If neither characteristic of the new creature is greater, evolve won’t trigger at all."
    );
    supported("Gluttonous Slug");
    // Gluttonous Slug: 0/3. Ornithopter (0/2): neither is greater. Memnite (1/1): power is.
    let mut t = TestGame::new(2);
    let slug = t.battlefield(P0, "Gluttonous Slug");
    enter(&mut t, P0, "Ornithopter");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
    enter(&mut t, P0, "Memnite");
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(t.counters(slug, counters::PLUS1), 1);
}
