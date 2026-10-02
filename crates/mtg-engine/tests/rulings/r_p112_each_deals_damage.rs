//! Rulings batch P112 — "each [creature] deals damage [equal to its power] to ...": every
//! one of those objects is a source dealing its own damage, all at the same time, and "its
//! power" is each source's own power (CR 120.2, 120.4, 608.2h). Regression tests for the
//! engine fix that made these effects deal damage from every source (before, one damage
//! event was dealt by the first of them, with the wrong amount), on every card the fix
//! changes.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s07_common::damage_on;
use crate::r_s13_common::add;
use crate::r_s19_common::add_lore;
use crate::r_s25_common::cast_new;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The sources of the damage events dealt to `to` this turn, in order.
fn damage_sources(t: &TestGame, to: ObjectId) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target: Entity::Object(o),
                amount,
                ..
            } if *o == to && *amount > 0 => Some(*source),
            _ => None,
        })
        .collect()
}

#[test]
fn bartz_and_boko_each_other_bird_deals_its_own_power() {
    cr!("120.2", "608.2h");
    supported("Bartz and Boko");
    let mut t = TestGame::new(2);
    let crow = t.battlefield(P0, "Storm Crow");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer_targets(P0, &[myr.into()]);
    t.enter(P0, "Bartz and Boko");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    // Storm Crow (1) and Vampire Nighthawk (2) — not Bartz and Boko (4) itself. Nighthawk
    // isn't a Bird: only the Crow deals damage.
    assert_eq!(damage_on(&t, myr), 1);
    assert_eq!(damage_sources(&t, myr), vec![crow]);
    let _ = hawk;
}

#[test]
fn kamahls_will_each_creature_deals_damage_equal_to_its_power() {
    cr!("120.2", "608.2h", "608.2b");
    ruling!(
        "Kamahl's Will",
        "If the target of the second mode becomes illegal, no damage will be dealt."
    );
    supported("Kamahl's Will");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Kamahl's Will");
    t.cast(P0, card).modes(&[1]).target(myr).go();
    t.resolve_all();
    assert_eq!(damage_on(&t, myr), 5);
    let mut srcs = damage_sources(&t, myr);
    srcs.sort();
    assert_eq!(srcs, vec![bears, giant]);
    // The target is gone: nothing is dealt.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Kamahl's Will");
    t.cast(P0, card).modes(&[1]).target(wurm).go();
    destroy(&mut t, wurm);
    t.resolve_all();
    assert!(t
        .g
        .turn_events
        .iter()
        .all(|e| !matches!(e, Event::Damage { .. })));
}

#[test]
fn nissas_judgment_support_first_then_each_countered_creature_deals_damage() {
    cr!("120.2", "608.2c", "701.41a");
    ruling!(
        "Nissa's Judgment",
        "You finish the support action before any creatures deal damage. Creatures that get a +1/+1 counter will deal damage to the creature an opponent controls, if applicable."
    );
    supported("Nissa's Judgment");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Craw Wurm"); // no counter: deals nothing
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer_targets(P0, &[bears.into(), giant.into()]);
    t.answer_targets(P0, &[myr.into()]);
    cast_new(&mut t, P0, "Nissa's Judgment", &[]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    // 3 + 4.
    assert_eq!(damage_on(&t, myr), 7);
}

/// P0's Grizzly Bears (the target), a Hill Giant with a +1/+1 counter (modified) and an
/// unmodified Craw Wurm; P1's Darksteel Myr. P0 casts Signature Slam; `spoil` happens in
/// response. Returns (game, bears, giant, myr).
fn signature_slam(spoil: Option<bool>) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported("Signature Slam");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, giant, counters::PLUS1, 1);
    t.battlefield(P0, "Craw Wurm");
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_new(&mut t, P0, "Signature Slam", &[bears.into(), myr.into()]);
    match spoil {
        // The creature you control is gone.
        Some(true) => destroy(&mut t, bears),
        // The creature you don't control is no longer a legal target (now yours).
        Some(false) => crate::r_s06_common::give_control(&mut t, myr, P0),
        None => {}
    }
    t.resolve_all();
    (t, bears, giant, myr)
}

#[test]
fn signature_slam_each_modified_creature_deals_its_power() {
    cr!("120.2", "608.2h", "700.9");
    let (t, bears, _, myr) = signature_slam(None);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // Bears 3 + Hill Giant 4; the Craw Wurm isn't modified.
    assert_eq!(damage_on(&t, myr), 7);
}

#[test]
fn signature_slam_with_one_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Signature Slam",
        "If the creature you control is an illegal target as Signature Slam tries to resolve but the creature you don't control is still a legal target, each modified creature you control (which may include the target creature you control if it's still on the battlefield) will still deal damage equal to its power to the creature you don't control."
    );
    ruling!(
        "Signature Slam",
        "If the creature you don't control is an illegal target as Signature Slam tries to resolve but the creature you control is a legal target, you'll still put a +1/+1 counter on the target creature you control."
    );
    let (t, _, giant, myr) = signature_slam(Some(true));
    assert_eq!(damage_on(&t, myr), 4);
    assert_eq!(damage_sources(&t, myr), vec![giant]);
    let (t, bears, _, myr) = signature_slam(Some(false));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(damage_on(&t, myr), 0);
}

#[test]
fn the_bears_of_littjara_each_big_creature_deals_its_power() {
    cr!("120.2", "608.2h", "714.2b");
    supported("The Bears of Littjara");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Bears of Littjara");
    add(&mut t, saga, counters::LORE, 2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    t.battlefield(P0, "Hill Giant"); // power 3: deals nothing
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer_targets(P0, &[myr.into()]);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(damage_on(&t, myr), 12);
    let mut srcs = damage_sources(&t, myr);
    srcs.sort();
    assert_eq!(srcs, vec![wurm, maw]);
}

#[test]
fn case_of_the_gateway_express_each_creature_deals_one() {
    cr!("120.2");
    supported("Case of the Gateway Express");
    let mut t = TestGame::new(2);
    for n in ["Grizzly Bears", "Hill Giant", "Craw Wurm"] {
        t.battlefield(P0, n);
    }
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer_targets(P0, &[myr.into()]);
    t.enter(P0, "Case of the Gateway Express");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(damage_on(&t, myr), 3);
    assert_eq!(damage_sources(&t, myr).len(), 3);
}

#[test]
fn sarkhan_the_masterless_each_dragon_deals_one() {
    cr!("120.2", "603.2");
    supported("Sarkhan the Masterless");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sarkhan the Masterless");
    t.battlefield(P1, "Shivan Dragon");
    t.battlefield(P1, "Shivan Dragon");
    let myr = t.battlefield(P0, "Darksteel Myr");
    attack_with(&mut t, &[(myr, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(damage_on(&t, myr), 2);
    assert_eq!(damage_sources(&t, myr).len(), 2);
}
