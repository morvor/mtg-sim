//! Rulings batch S08 — fight (CR 701.14): "Each deals damage equal to its power to the
//! other." A targeted fight whose target has become illegal does nothing (CR 608.2b).

use crate::r_s01_common::*;
use crate::r_s06_common::*;
use crate::r_s08_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_fight_with_an_opponents_creature_gaining_hexproof_does_nothing() {
    cr!("701.14a", "701.14b", "608.2b", "702.11b");
    ruling!(
        "Triangle of War",
        "If either target is illegal when the ability resolves, it will do nothing."
    );
    supported("Triangle of War");
    supported("Blossoming Defense");
    // Triangle of War: "{2}, Sacrifice this artifact: Target creature you control fights
    // target creature an opponent controls."
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let triangle = t.battlefield(P0, "Triangle of War");
        let wurm = t.battlefield(P0, "Craw Wurm");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", 2);
        t.activate(P0, triangle, 0, &[Entity::Object(wurm), Entity::Object(bears)])
            .unwrap();
        if respond {
            // P1 gives its Bears +2/+2 and hexproof: an illegal target.
            t.lands(P1, "Forest", 1);
            let defense = t.hand(P1, "Blossoming Defense");
            t.cast(P1, defense).target(bears).go();
            t.resolve();
            assert_eq!(t.pt(bears), (4, 4));
        }
        t.resolve_all();
        if respond {
            // Neither creature dealt damage: the legal Wurm didn't fight alone.
            assert!(t.on_battlefield(bears));
            assert_eq!(t.obj_now(bears).damage, 0);
            assert_eq!(t.obj_now(wurm).damage, 0);
        } else {
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert_eq!(t.obj_now(wurm).damage, 2);
        }
    }
}

#[test]
fn a_fight_whose_own_creature_stops_matching_the_target_does_nothing() {
    cr!("701.14b", "608.2b", "115.1c");
    ruling!(
        "Contested Cliffs",
        "If either target is illegal when the ability resolves, it will do nothing."
    );
    supported("Contested Cliffs");
    supported("Amphibian Downpour");
    // Contested Cliffs: "{R}{G}, {T}: Target Beast creature you control fights target
    // creature an opponent controls." P1 responds by turning the Beast into a 1/1 Frog
    // with Amphibian Downpour: it's no longer a Beast.
    let mut t = TestGame::new(2);
    let cliffs = t.battlefield(P0, "Contested Cliffs");
    let baloth = t.battlefield(P0, "Enormous Baloth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.answer_targets(P0, &[Entity::Object(baloth)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let fight = activate_containing(&mut t, P0, cliffs, "fights");
    assert!(fight.is_ok(), "{fight:?}");
    t.lands(P1, "Island", 3);
    let downpour = t.hand(P1, "Amphibian Downpour");
    t.cast(P1, downpour).target(baloth).go();
    // Its storm trigger (no copies), then the Aura resolve; the fight is still waiting.
    while t.stack_len() > 1 {
        t.resolve();
    }
    assert_eq!(t.pt(baloth), (1, 1));
    assert!(!t.obj_now(baloth).chars.has_subtype("Beast"));
    t.resolve_all();
    // The Frog didn't deal damage to the Bears, and the Bears didn't deal damage to it.
    assert!(t.on_battlefield(baloth));
    assert_eq!(t.obj_now(baloth).damage, 0);
    assert_eq!(t.obj_now(bears).damage, 0);
}
