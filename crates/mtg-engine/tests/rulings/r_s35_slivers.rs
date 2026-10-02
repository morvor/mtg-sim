//! Rulings batch S35 — Slivers: abilities and power/toughness boosts that several Slivers
//! grant are cumulative (CR 613.4c, 113.2c), though more than one instance of some
//! abilities (flying, menace) is redundant (CR 702.9c, 702.111c).

use crate::r_s01_common::{attack_with, supported};
use crate::r_s21_common::legal_blocks;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn two_muscle_slivers_give_plus_two_but_two_flying_abilities_are_one() {
    cr!("613.4c", "702.9b", "702.9c");
    ruling!(
        "Muscle Sliver",
        "Abilities that Slivers grant, as well as power/toughness boosts, are cumulative. However, for some abilities, like flying, having more than one instance of the ability doesn't provide any additional benefit."
    );
    supported("Muscle Sliver");
    supported("Galerider Sliver");
    // Muscle Sliver: "Sliver creatures you control get +1/+1." Each of two Muscle Slivers
    // (1/1) gets +2/+2.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Muscle Sliver");
    let b = t.battlefield(P0, "Muscle Sliver");
    t.g.recompute();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
    // Galerider Sliver: "Sliver creatures you control have flying." With two, a Sliver has
    // flying twice, which is no better than once: a creature with reach can still block
    // it, one without flying or reach still can't.
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Galerider Sliver");
    t.battlefield(P0, "Galerider Sliver");
    let spider = t.battlefield(P1, "Giant Spider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    let flying = t
        .obj_now(g1)
        .chars
        .keywords()
        .filter(|k| k.kind == mtg_engine::keywords::KeywordKind::Flying)
        .count();
    assert_eq!(flying, 2);
    attack_with(&mut t, &[(g1, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(spider, g1)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, g1)]));
}

#[test]
fn two_leeching_slivers_trigger_twice_but_two_menaces_are_one() {
    cr!("113.2c", "702.111b", "702.111c");
    ruling!(
        "Leeching Sliver",
        "Abilities that Slivers grant, as well as power/toughness boosts, are cumulative. However, for some abilities, like indestructible and the ability granted by Belligerent Sliver, having more than one instance of the ability doesn’t provide any additional benefit."
    );
    supported("Leeching Sliver");
    supported("Belligerent Sliver");
    // Leeching Sliver: "Whenever a Sliver you control attacks, defending player loses 1
    // life." With two, one attacking Sliver makes P1 lose 2 life.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Leeching Sliver");
    t.battlefield(P0, "Leeching Sliver");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Belligerent Sliver: "Sliver creatures you control have menace." With two, an
    // attacking Sliver still can be blocked by two creatures (not four), not by one.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Belligerent Sliver");
    t.battlefield(P0, "Belligerent Sliver");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(b1, s)]));
    assert!(legal_blocks(&mut t, P1, &[(b1, s), (b2, s)]));
}
