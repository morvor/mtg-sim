//! Rulings batch P063 — "French vanilla" permanents and Equipment (their types, colors,
//! landwalk, haste from Equipment), Obsidian Fireheart's blaze counters (an ability that
//! lasts for as long as a counter is on the land, CR 611.2b), and Rite of Passage
//! ("whenever a creature you control is dealt damage").

use crate::r_p063_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, target_candidates};
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, attached_to, give_control};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn lucent_liminid_is_affected_by_what_affects_creatures_and_enchantments() {
    cr!("205.2a", "300.2");
    ruling!(
        "Lucent Liminid",
        "Lucent Liminid is an enchantment creature, in much the same way that Ornithopter is an artifact creature. It’s affected by anything that affects creatures and anything that affects enchantments."
    );
    supported("Lucent Liminid");
    // Naturalize (artifact or enchantment) and Shock (creature) can both target it.
    let mut t = TestGame::new(2);
    let lim = t.battlefield(P1, "Lucent Liminid");
    crate::r_s29_common::cast_and_resolve(&mut t, P0, "Naturalize", &[obj(lim)]);
    assert!(t.in_graveyard(P1, "Lucent Liminid"));
    let mut t = TestGame::new(2);
    let lim = t.battlefield(P1, "Lucent Liminid");
    crate::r_s29_common::cast_and_resolve(&mut t, P0, "Shock", &[obj(lim)]);
    assert_eq!(t.obj_now(lim).damage, 2);
    assert!(t.obj_now(lim).is(CardType::Creature) && t.obj_now(lim).is(CardType::Enchantment));
}

#[test]
fn skyshroud_falcon_is_a_bird_scion_of_ugin_is_colorless() {
    cr!("205.3a", "105.2c", "105.4");
    ruling!(
        "Skyshroud Falcon",
        "No it is not a Falcon creature type. It is a Bird."
    );
    ruling!(
        "Scion of Ugin",
        "Scion of Ugin is colorless. It isn’t monocolored, nor is it an artifact."
    );
    supported("Skyshroud Falcon");
    supported("Scion of Ugin");
    let mut t = TestGame::new(2);
    let falcon = t.battlefield(P0, "Skyshroud Falcon");
    let subs: Vec<String> = t.obj_now(falcon).chars.subtypes.iter().map(|s| s.to_string()).collect();
    assert_eq!(subs, ["Bird"]);
    let scion = t.battlefield(P0, "Scion of Ugin");
    let o = t.obj_now(scion);
    assert!(crate::r_s05_common::colorless(&t, scion), "colorless");
    assert!(!o.is(CardType::Artifact));
    // Not an artifact: Shatter can't target it.
    let from = t.asked().len();
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Shatter");
    let shatter = t.hand(P0, "Shatter");
    let _ = t.cast(P0, shatter).try_go();
    assert!(target_candidates(&t, P0, from)
        .iter()
        .all(|c| !c.contains(&obj(scion))));
}

#[test]
fn alexis_cloak_leaves_auras_already_on_the_creature() {
    cr!("702.18a", "303.4d", "704.5m");
    ruling!("Alexi's Cloak", "Any Auras already on the creature remain.");
    supported("Alexi's Cloak");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = attach_new(&mut t, P1, "Holy Strength", bears);
    crate::r_s29_common::cast_and_resolve(&mut t, P0, "Alexi's Cloak", &[obj(bears)]);
    assert!(has(&mut t, bears, KeywordKind::Shroud));
    assert_eq!(attached_to(&t, strength), Some(obj(bears)), "Holy Strength stays");
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn dryads_favor_forestwalk_cares_about_the_forest_land_type() {
    cr!("702.14a", "702.14c", "305.6");
    ruling!(
        "Dryad's Favor",
        "Forestwalk cares about lands with the land type Forest, not necessarily lands named Forest."
    );
    supported("Dryad's Favor");
    for (land, walks) in [("Overgrown Tomb", true), ("Mountain", false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Dryad's Favor", bears);
        t.battlefield(P1, land);
        let wall = t.battlefield(P1, "Wall of Wood");
        crate::r_s03_common::to_blockers(
            &mut t,
            &[(bears, Entity::Player(P1))],
            &[(wall, bears)],
        );
        let blocked = crate::r_p076_common::is_blocked(&t, bears);
        assert_eq!(blocked, !walks, "{land}");
        finish_combat(&mut t);
        assert_eq!(t.life(P1), if walks { 18 } else { 20 }, "{land}");
    }
}

#[test]
fn haste_equipment_moved_away_before_attacking_leaves_the_creature_unable_to_attack() {
    cr!("302.6", "702.10b", "301.5c");
    ruling!(
        "Crystal Slipper",
        "If a creature enters the battlefield under your control and gains haste, but then loses it before attacking, it won't be able to attack. This means that you can't use one Crystal Slipper to allow two new creatures to attack in the same combat."
    );
    ruling!(
        "Strider Harness",
        "If a creature enters the battlefield under your control and gains haste, but then loses it before attacking, it won’t be able to attack that turn. This means that you can’t use one Strider Harness to allow two new creatures to attack in the same turn."
    );
    for eq in ["Crystal Slipper", "Strider Harness"] {
        supported(eq);
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, eq);
        let a = t.battlefield_sick(P0, "Grizzly Bears");
        let b = t.battlefield_sick(P0, "Hill Giant");
        pool(&mut t, P0, ManaType::C, 2);
        t.activate(P0, e, 0, &[obj(a)]).unwrap();
        t.resolve_all();
        assert!(has(&mut t, a, KeywordKind::Haste), "{eq}");
        t.activate(P0, e, 0, &[obj(b)]).unwrap();
        t.resolve_all();
        assert_eq!(attached_to(&t, e), Some(obj(b)));
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!can_attack(&mut t, a), "{eq}: the first creature lost haste");
        assert!(can_attack(&mut t, b), "{eq}");
    }
}

/// P0's Obsidian Fireheart (with {1}{R}{R} in the pool) and P1's Mountain.
fn fireheart(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Obsidian Fireheart");
    let fh = t.battlefield(P0, "Obsidian Fireheart");
    let land = t.battlefield(P1, "Mountain");
    (fh, land)
}

fn blaze(t: &mut TestGame, fh: ObjectId, land: ObjectId) {
    pool(t, P0, ManaType::R, 3);
    t.activate(P0, fh, 0, &[obj(land)]).unwrap();
}

#[test]
fn obsidian_fireheart_activated_twice_in_response_the_second_fizzles() {
    cr!("608.2b", "115.1");
    ruling!(
        "Obsidian Fireheart",
        "A land with a blaze counter on it is an illegal target for this ability. You may activate the ability targeting a land, then activate the ability in response to itself targeting the same land."
    );
    let mut t = TestGame::new(2);
    let (fh, land) = fireheart(&mut t);
    blaze(&mut t, fh, land);
    blaze(&mut t, fh, land);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.counters(land, "blaze"), 1);
    t.resolve();
    assert_eq!(t.counters(land, "blaze"), 1, "the second one didn't resolve");
    // Now the land can't be targeted.
    pool(&mut t, P0, ManaType::R, 3);
    assert!(t.activate(P0, fh, 0, &[obj(land)]).is_err());
}

#[test]
fn obsidian_fireheart_the_land_burns_after_the_fireheart_leaves() {
    cr!("611.2b", "603.2");
    ruling!(
        "Obsidian Fireheart",
        "whether the targeted land has the triggered ability that’s been granted to it depends only on whether it has a blaze counter on it, not on whether Obsidian Fireheart is still on the battlefield"
    );
    let mut t = TestGame::new(2);
    let (fh, land) = fireheart(&mut t);
    blaze(&mut t, fh, land);
    t.resolve_all();
    move_to(&mut t, fh, Zone::Graveyard(P0));
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn obsidian_fireheart_more_blaze_counters_still_one_damage() {
    cr!("611.2b", "122.1");
    ruling!(
        "Obsidian Fireheart",
        "If a land ends up with more than one blaze counter on it (thanks to Doubling Season or Gilder Bairn, for example), the ability still only causes it to deal 1 damage to its controller each turn."
    );
    let mut t = TestGame::new(2);
    let (fh, land) = fireheart(&mut t);
    blaze(&mut t, fh, land);
    t.resolve_all();
    put(&mut t, land, "blaze", 2);
    assert_eq!(t.counters(land, "blaze"), 3);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn obsidian_fireheart_ability_doesnt_follow_moved_blaze_counters() {
    cr!("611.2b", "122.5");
    ruling!(
        "Obsidian Fireheart",
        "If all blaze counters on a land are moved to a different land, the triggered ability doesn’t follow them."
    );
    let mut t = TestGame::new(2);
    let (fh, land) = fireheart(&mut t);
    let other = t.battlefield(P1, "Forest");
    blaze(&mut t, fh, land);
    t.resolve_all();
    // The counter moves from the Mountain to the Forest.
    assert_eq!(t.g.remove_counters(obj(land), "blaze", 1), 1);
    put(&mut t, other, "blaze", 1);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "neither land deals damage");
    // Control: with the counter left in place, the Mountain burns P1.
    let mut t = TestGame::new(2);
    let (fh, land) = fireheart(&mut t);
    blaze(&mut t, fh, land);
    t.resolve_all();
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn obsidian_fireheart_land_burns_its_controller_in_their_upkeep() {
    cr!("603.2", "109.5", "611.2b");
    ruling!(
        "Obsidian Fireheart",
        "The ability gained by the land triggers at the beginning of the upkeep of the land’s controller, not the beginning of the upkeep of Obsidian Fireheart’s controller. The player who controls the land at the time the ability triggers is the one who’s dealt damage."
    );
    let mut t = TestGame::new(2);
    let (fh, land) = fireheart(&mut t);
    blaze(&mut t, fh, land);
    t.resolve_all();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19), "only in P1's upkeep");
    // P0 gains control of the land: it burns P0, in P0's upkeep.
    give_control(&mut t, land, P0);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19));
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (19, 19));
}

#[test]
fn rite_of_passage_one_counter_for_damage_from_several_sources_at_once() {
    cr!("510.2", "603.2c", "120.3");
    ruling!(
        "Rite of Passage",
        "If the creature is dealt damage by more than one source at the same time, it gets only one counter."
    );
    supported("Rite of Passage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rite of Passage");
    let giant = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    to_first_strike_damage(&mut t, &[giant], &[(a, giant), (b, giant)]);
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.counters(giant, PLUS1), 1);
}

#[test]
fn rite_of_passage_no_counter_if_all_the_damage_is_prevented() {
    cr!("615.1", "702.16e", "603.2");
    ruling!(
        "Rite of Passage",
        "The creature doesn’t get a counter if all the damage that would be dealt to it is prevented."
    );
    // Paladin en-Vec has protection from red: Shock's damage is prevented.
    for (name, n) in [("Paladin en-Vec", 0), ("Hill Giant", 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rite of Passage");
        let c = t.battlefield(P0, name);
        let shock = t.hand(P1, "Shock");
        pool(&mut t, P1, ManaType::R, 1);
        // (Protection stops Shock targeting it: the damage comes from an untargeted source.)
        crate::r_s06_common::damage(&mut t, shock, 2, c);
        t.resolve_all();
        assert_eq!(t.counters(c, PLUS1), n, "{name}");
    }
}
