//! Rulings batch P102 — untap restrictions (Winter Orb, Winter Moon with Static Orb,
//! Hokori, Dovin Baan's emblem, Winter's Night; CR 502.3), Quicksilver Fountain's flood
//! counters, and Thieves' Auction.

use crate::r_p062_common::{is_tapped, tap};
use crate::r_p102_common::*;
use crate::r_s01_common::watch;
use crate::r_s04_common::next_upkeep;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn tapped_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n)
        .map(|_| {
            let id = t.battlefield(p, name);
            tap(t, id);
            id
        })
        .collect()
}

fn untapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| !is_tapped(t, **id)).count()
}

#[test]
fn winter_orb_cant_be_tapped_at_will() {
    cr!("502.3", "602.1");
    ruling!(
        "Winter Orb",
        "You can’t tap Winter Orb just because you want to."
    );
    supported("Winter Orb");
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Winter Orb");
    lots(&mut t, P0);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, orb));
    let lands = tapped_n(&mut t, P0, "Forest", 3);
    next_upkeep(&mut t, P0);
    assert_eq!(untapped(&t, &lands), 1);
    // Tapped by another effect, it stops restricting.
    tap(&mut t, orb);
    next_upkeep(&mut t, P0);
    assert_eq!(untapped(&t, &lands), 3);
}

#[test]
fn winter_moon_and_static_orb_are_cumulative() {
    cr!("502.3");
    ruling!(
        "Winter Moon",
        "If Winter Moon is on the battlefield while another effect restricts untapping lands or permanents in a different way, their effects are cumulative."
    );
    supported("Winter Moon");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Winter Moon");
    t.battlefield(P1, "Static Orb");
    let pools = tapped_n(&mut t, P0, "Breeding Pool", 3);
    let forests = tapped_n(&mut t, P0, "Forest", 2);
    let bears = tapped_n(&mut t, P0, "Grizzly Bears", 2);
    next_upkeep(&mut t, P0);
    let all: Vec<ObjectId> = [pools.clone(), forests, bears].concat();
    assert_eq!(untapped(&t, &all), 2);
    assert!(untapped(&t, &pools) <= 1);
}

#[test]
fn multiple_winter_moons_are_redundant() {
    cr!("502.3");
    ruling!(
        "Winter Moon",
        "If multiple Winter Moons are on the battlefield, their effects are redundant."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Winter Moon");
    t.battlefield(P1, "Winter Moon");
    let pools = tapped_n(&mut t, P0, "Breeding Pool", 3);
    let forests = tapped_n(&mut t, P0, "Forest", 2);
    next_upkeep(&mut t, P0);
    assert_eq!(untapped(&t, &pools), 1);
    assert_eq!(untapped(&t, &forests), 2);
}

#[test]
fn two_hokoris_untap_two_lands() {
    cr!("502.3", "603.2");
    ruling!(
        "Hokori, Dust Drinker",
        "If there are two Hokori on the battlefield (for example, if Mirror Gallery is on the battlefield), each player untaps two lands they control at the beginning of their upkeep."
    );
    supported("Hokori, Dust Drinker");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mirror Gallery");
    t.battlefield(P1, "Hokori, Dust Drinker");
    t.battlefield(P1, "Hokori, Dust Drinker");
    assert_eq!(t.named_on_battlefield("Hokori, Dust Drinker").len(), 2);
    let lands = tapped_n(&mut t, P0, "Forest", 3);
    choose(&mut t, P0, &[lands[0]]);
    choose(&mut t, P0, &[lands[1]]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(untapped(&t, &lands), 2);
}

/// P0's Dovin Baan with `loyalty` loyalty.
fn dovin(t: &mut TestGame, loyalty: u32) -> ObjectId {
    let d = t.battlefield(P0, "Dovin Baan");
    t.g.objects[d.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), loyalty);
    d
}

#[test]
fn dovin_baan_emblem_opponent_chooses_what_to_untap() {
    cr!("502.3", "114.4");
    ruling!(
        "Dovin Baan",
        "An opponent affected by Dovin Baan's emblem chooses which two of their permanents to untap, not you."
    );
    let mut t = TestGame::new(2);
    let d = dovin(&mut t, 7);
    t.activate(P0, d, 2, &[]).unwrap();
    t.resolve_all();
    let mine = tapped_n(&mut t, P0, "Forest", 3);
    let theirs = tapped_n(&mut t, P1, "Forest", 3);
    let bears = tapped_n(&mut t, P1, "Grizzly Bears", 1);
    choose(&mut t, P1, &[theirs[0], bears[0]]);
    let from = t.asked().len();
    next_upkeep(&mut t, P1);
    let askers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(askers, vec![P1]);
    assert!(!is_tapped(&t, theirs[0]));
    assert!(!is_tapped(&t, bears[0]));
    assert_eq!(untapped(&t, &theirs), 1);
    // P0 isn't affected.
    next_upkeep(&mut t, P0);
    assert_eq!(untapped(&t, &mine), 3);
}

#[test]
fn dovin_baan_plus_one_targets_any_creature_or_none() {
    cr!("115.1d", "602.5b");
    ruling!(
        "Dovin Baan",
        "Dovin Baan's first ability can target any creature, not just one with an activated ability. It can also be activated targeting no creature at all."
    );
    let mut t = TestGame::new(2);
    let d = dovin(&mut t, 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, d, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (-1, 2));
    assert_eq!(t.counters(d, counters::LOYALTY), 4);
    let mut t = TestGame::new(2);
    let d = dovin(&mut t, 3);
    t.answer_targets(P0, &[]);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(d, counters::LOYALTY), 4);
}

/// P0 taps a Snow-Covered Forest for mana under Winter's Night (choosing no extra mana
/// beyond what it can).
fn winters_night_tap(t: &mut TestGame) -> ObjectId {
    supported("Winter's Night");
    t.battlefield(P1, "Winter's Night");
    let land = t.battlefield(P0, "Snow-Covered Forest");
    assert!(tap_for_mana(t, P0, land, ""));
    t.settle();
    assert!(is_tapped(t, land));
    land
}

#[test]
fn winters_night_does_nothing_to_an_untapped_land() {
    cr!("502.3", "611.2a");
    ruling!(
        "Winter's Night",
        "If a land affected by Winter's Night is untapped at the time its controller's next untap step begins, the \"doesn't untap\" effect doesn't do anything."
    );
    let mut t = TestGame::new(2);
    let land = winters_night_tap(&mut t);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2, "an extra mana");
    // Untapped before the next untap step: nothing happens then, and the effect is used up.
    crate::r_p021_common::untap(&mut t, land);
    next_upkeep(&mut t, P0);
    assert!(!is_tapped(&t, land));
    tap(&mut t, land);
    next_upkeep(&mut t, P0);
    assert!(!is_tapped(&t, land));
}

#[test]
fn winters_night_follows_the_land_to_a_new_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Winter's Night",
        "Winter's Night doesn't track the lands' controllers."
    );
    let mut t = TestGame::new(2);
    let land = winters_night_tap(&mut t);
    crate::r_p015_common::gain_control(&mut t, P1, land);
    assert_eq!(t.obj_now(land).controller, P1);
    next_upkeep(&mut t, P1);
    assert!(is_tapped(&t, land), "P1's next untap step");
    next_upkeep(&mut t, P1);
    assert!(!is_tapped(&t, land));
}

#[test]
fn quicksilver_fountain_flood_counters_outlast_it() {
    cr!("611.2b", "613.7a", "122.1");
    ruling!(
        "Quicksilver Fountain",
        "The land stays an Island until the flood counter is removed, even if Quicksilver Fountain leaves the battlefield."
    );
    supported("Quicksilver Fountain");
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P1, "Quicksilver Fountain");
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[o(forest)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(forest, "flood"), 1);
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    assert!(!t.obj_now(forest).chars.has_subtype("Forest"));
    crate::r_s02_common::destroy(&mut t, fountain);
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    t.g.objects[forest.0 as usize].counters.remove("flood");
    t.g.recompute();
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
}

#[test]
fn quicksilver_fountain_effects_get_their_timestamps_on_resolution() {
    cr!("613.7a", "613.7e");
    ruling!(
        "Quicksilver Fountain",
        "The timestamp of the land-type-changing ability is set when the triggered ability resolves (so each one has a different timestamp)."
    );
    // P0's Forest is flooded (an Island), then Evil Presence (later) makes it a Swamp. The
    // next upkeep it's a non-Island land again: a new flood counter, a newer effect: an
    // Island again.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Quicksilver Fountain");
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[o(forest)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    crate::r_s06_common::attach_new(&mut t, P1, "Evil Presence", forest);
    assert!(t.obj_now(forest).chars.has_subtype("Swamp"));
    assert!(!t.obj_now(forest).chars.has_subtype("Island"));
    t.answer_targets(P0, &[o(forest)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(forest, "flood"), 2);
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    assert!(!t.obj_now(forest).chars.has_subtype("Swamp"));
}

#[test]
fn thieves_auction_auras_dont_target_and_triggers_wait() {
    cr!("303.4f", "603.3", "608.2c");
    ruling!(
        "Thieves' Auction",
        "An Aura put onto the battlefield this way does not target the permanent it is placed upon."
    );
    ruling!(
        "Thieves' Auction",
        "Triggers from the permanents leaving the battlefield and then being put back onto the battlefield are not put on the stack until the entire effect is done."
    );
    supported("Thieves' Auction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let visionary = t.battlefield(P1, "Elvish Visionary");
    let pacifism = crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", bears);
    lots(&mut t, P0);
    let ta = t.hand(P0, "Thieves' Auction");
    t.cast(P0, ta).go();
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        // (The resolving Auction itself is still on the stack.)
        |g| {
            g.stack
                .iter()
                .filter(|s| g.obj(**s).chars.name != "Thieves' Auction")
                .count()
        },
    );
    // P0 takes Bears, P1 the Bogle, P0 Pacifism (enchanting P1's hexproof Bogle), P1 the
    // Visionary.
    t.answer_choose_moved(P0, &[o(bears)]);
    t.answer_choose_moved(P1, &[o(bogle)]);
    t.answer_choose_moved(P0, &[o(pacifism)]);
    t.answer_choose_moved(P0, &[o(bogle)]);
    t.answer_choose_moved(P1, &[o(visionary)]);
    t.resolve();
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|n| *n == 0), "{seen:?}");
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(bogle).controller, P1);
    assert_eq!(t.obj_now(pacifism).controller, P0);
    assert!(t.obj_now(bogle).tapped);
    assert_eq!(
        crate::r_s06_common::attached_to(&t, pacifism),
        Some(o(t.g.current(bogle)))
    );
    // The Visionary's "draw a card" trigger is on the stack now.
    assert_eq!(t.zone(visionary), Zone::Battlefield);
    assert_eq!(t.stack_len(), 1);
}
