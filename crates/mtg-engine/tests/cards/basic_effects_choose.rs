//! Tapping, sacrificing, and exiling objects chosen as the effect happens (CR 608.2d):
//! "any number of", "up to N", "one or more"; optional actions that are costs paid on
//! resolution (CR 118.12); and later sentences about those objects ("for each creature
//! tapped this way", "that much damage", "return those cards ...").

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

#[test]
fn marshaling_the_troops_gains_life_for_each_creature_tapped() {
    cr!("608.2d", "701.26a");
    assert_supported("Marshaling the Troops");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(c);
    t.lands(P0, "Forest", 2);
    let spell = t.hand(P0, "Marshaling the Troops");
    t.answer_choose(P0, &objs(&[a, b]));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.life(P0), 28);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
}

#[test]
fn devout_invocation_counts_only_the_creatures_actually_tapped() {
    cr!("608.2d");
    assert_supported("Devout Invocation");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 7);
    let spell = t.hand(P0, "Devout Invocation");
    t.answer_choose(P0, &objs(&[a]));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 1);
}

#[test]
fn raff_taps_two_creatures_only_if_it_can() {
    cr!("118.12");
    assert_supported("Raff, Weatherlight Stalwart");
    for creatures in [1, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Raff, Weatherlight Stalwart");
        for _ in 1..creatures {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.lands(P0, "Mountain", 1);
        let shock = t.hand(P0, "Shock");
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.cast(P0, shock).target(P1).go();
        t.resolve_all();
        // Shock left the hand; a card was drawn only if two creatures could be tapped.
        let drew = creatures == 2;
        assert_eq!(t.hand_size(P0), hand - 1 + drew as usize, "{creatures}");
    }
}

#[test]
fn reservoir_kraken_an_opponent_may_tap_a_creature() {
    cr!("118.12");
    assert_supported("Reservoir Kraken");
    let mut t = TestGame::new(2);
    let kraken = t.battlefield(P0, "Reservoir Kraken");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(t.obj_now(kraken).tapped);
    assert_eq!(t.named_on_battlefield("Fish Token").len(), 1);
}

#[test]
fn last_ditch_effort_deals_damage_equal_to_the_number_sacrificed() {
    cr!("701.21a", "608.2d");
    assert_supported("Last-Ditch Effort");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Last-Ditch Effort");
    t.cast(P0, spell).target(P1).go();
    t.answer_choose(P0, &objs(&[a, b]));
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn renounce_gains_life_for_each_permanent_sacrificed() {
    cr!("701.21a");
    assert_supported("Renounce");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Plains", 4);
    let spell = t.hand(P0, "Renounce");
    t.answer_choose(P0, &objs(&lands[2..]));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn ravenous_rotbelly_opponents_sacrifice_that_many() {
    cr!("603.12", "701.21a");
    assert_supported("Ravenous Rotbelly");
    let mut t = TestGame::new(2);
    let z1 = t.battlefield(P0, "Walking Corpse");
    let z2 = t.battlefield(P0, "Walking Corpse");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[z1, z2]));
    t.enter(P0, "Ravenous Rotbelly");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Walking Corpse").len(), 0);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn yorion_returns_the_exiled_permanents_at_the_next_end_step() {
    cr!("603.7c", "701.13a");
    assert_supported("Yorion, Sky Nomad");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &objs(&[bears]));
    t.enter(P0, "Yorion, Sky Nomad");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn nahiris_resolve_returns_them_at_your_next_upkeep() {
    cr!("603.7c");
    assert_supported("Nahiri's Resolve");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nahiri's Resolve");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &objs(&[bears]));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Not at the opponent's upkeep.
    t.advance_to(P1, Step::Draw);
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn lumbering_battlement_exiles_until_it_leaves() {
    cr!("610.3");
    assert_supported("Lumbering Battlement");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &objs(&[bears]));
    let lb = t.enter(P0, "Lumbering Battlement");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.pt(lb), (6, 7));
    let lb = t.g.current(lb);
    t.g.destroy(lb, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn wormfang_newt_exiles_a_land_you_control() {
    cr!("608.2d");
    assert_supported("Wormfang Newt");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Island");
    t.battlefield(P1, "Island");
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.enter(P0, "Wormfang Newt");
    t.resolve_all();
    assert!(t.in_exile("Island"));
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
}

#[test]
fn urge_to_feed_counters_on_each_vampire_tapped() {
    cr!("608.2d");
    assert_supported("Urge to Feed");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vampire Interloper");
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Urge to Feed");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[v]));
    t.cast(P0, spell).target(target).go();
    t.resolve();
    assert!(t.obj_now(v).tapped);
    assert_eq!(t.counters(v, "+1/+1"), 1);
}

#[test]
fn gut_sacrifices_another_creature_or_an_artifact() {
    cr!("701.21a");
    assert_supported("Gut, True Soul Zealot");
    let mut t = TestGame::new(2);
    let gut = t.battlefield(P0, "Gut, True Soul Zealot");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[bears]));
    t.attack(&[(gut, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Skeleton Token").len(), 1);
}

#[test]
fn mana_seism_adds_mana_for_each_land_sacrificed() {
    cr!("701.21a", "106.4");
    assert_supported("Mana Seism");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Mana Seism");
    // {1}{R}: two Mountains pay for it; the other two are sacrificed.
    t.cast(P0, spell).go();
    let lands_left: Vec<ObjectId> = lands
        .iter()
        .copied()
        .filter(|l| !t.obj_now(*l).tapped)
        .collect();
    assert_eq!(lands_left.len(), 2);
    t.answer_choose(P0, &objs(&lands_left));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 2);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}
