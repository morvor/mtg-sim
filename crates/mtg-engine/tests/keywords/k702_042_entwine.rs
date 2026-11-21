//! CR 702.42 Entwine.

use crate::common_k702_011_017::assert_supported;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

fn entwine_offered(t: &TestGame) -> bool {
    t.asked().iter().any(|(_, d)| {
        matches!(d, Decision::OptionalCost { name, .. } if name == "entwine")
    })
}

#[test]
fn paying_the_entwine_cost_chooses_all_modes() {
    cr!("702.42", "702.42a");
    assert_supported("Barbed Lightning");
    let mut t = TestGame::new(2);
    // Barbed Lightning {2}{R}, entwine {2}: 3 damage to target creature and 3 damage to
    // target player.
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let bl = t.hand(P0, "Barbed Lightning");
    t.cast(P0, bl)
        .kicked(true)
        .target(giant)
        .target(P1)
        .go();
    assert!(entwine_offered(&t));
    // All five lands were tapped: {2}{R} plus {2}.
    assert!(t.g.permanents().all(|o| !o.chars.is_land() || o.tapped));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn without_entwine_only_the_specified_number_of_modes_is_chosen() {
    cr!("702.42a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let bl = t.hand(P0, "Barbed Lightning");
    t.cast(P0, bl).kicked(false).modes(&[1]).target(P1).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.life(P1), 17);
    // Two lands untapped: only {2}{R} was paid.
    assert_eq!(
        t.g.permanents().filter(|o| o.chars.is_land() && !o.tapped).count(),
        2
    );
}

#[test]
fn the_total_cost_including_entwine_must_be_paid() {
    cr!("702.42a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let bl = t.hand(P0, "Barbed Lightning");
    // {2}{R} plus {2} can't be paid with three lands: the casting is illegal.
    assert!(t
        .cast(P0, bl)
        .kicked(true)
        .target(giant)
        .target(P1)
        .try_go()
        .is_err());
    t.clear_answers();
    t.cast(P0, bl).kicked(false).modes(&[0]).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn an_entwined_spell_needs_targets_for_every_mode() {
    cr!("702.42a");
    let mut t = TestGame::new(2);
    // No creature to target: Barbed Lightning can't be entwined.
    t.lands(P0, "Mountain", 5);
    let bl = t.hand(P0, "Barbed Lightning");
    assert!(t.cast(P0, bl).kicked(true).target(P1).try_go().is_err());
    t.clear_answers();
    t.cast(P0, bl).kicked(false).modes(&[1]).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn an_entwine_cost_can_be_a_non_mana_cost() {
    cr!("702.42a");
    assert_supported("Solar Tide");
    let mut t = TestGame::new(2);
    // Solar Tide: destroy all creatures with power 2 or less / 3 or greater; entwine—
    // sacrifice two lands.
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 8);
    let tide = t.hand(P0, "Solar Tide");
    t.cast(P0, tide).kicked(true).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.graveyard_size(P0), 3); // two lands and Solar Tide
}

#[test]
fn entwined_modes_are_followed_in_the_order_written() {
    cr!("702.42b");
    assert_supported("Dream's Grip");
    let mut t = TestGame::new(2);
    // Dream's Grip: tap target permanent / untap target permanent; entwine {1}. Both
    // modes target the same untapped creature: it's tapped, then untapped.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let dg = t.hand(P0, "Dream's Grip");
    t.cast(P0, dg).kicked(true).target(bears).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, events::Event::Tapped { .. })));
}

#[test]
fn entwined_tokens_are_affected_by_a_later_mode() {
    cr!("702.42b");
    ruling!(
        "Goblin War Party",
        "If Goblin War Party is entwined, its modes are performed in order. The Goblin tokens you create get +1/+1 and gain haste."
    );
    assert_supported("Goblin War Party");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let gwp = t.hand(P0, "Goblin War Party");
    t.cast(P0, gwp).kicked(true).go();
    t.resolve_all();
    let goblins: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Goblin"))
        .map(|o| o.id)
        .collect();
    assert_eq!(goblins.len(), 3);
    for g in goblins {
        assert_eq!(t.pt(g), (2, 2));
        assert!(t.obj_now(g).has_keyword(keywords::KeywordKind::Haste));
    }
}

#[test]
fn entwine_chooses_every_mode_of_a_choose_two_spell() {
    cr!("702.42a", "702.42b");
    ruling!(
        "Kaya's Guile",
        "A creature sacrificed for the first mode of Kaya’s Guile will be exiled if it ends up in an opponent’s graveyard and the second mode was also chosen."
    );
    assert_supported("Kaya's Guile");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Plains", 5);
    t.lands(P0, "Swamp", 2);
    let kg = t.hand(P0, "Kaya's Guile");
    t.cast(P0, kg).kicked(true).go();
    t.resolve_all();
    // Each opponent sacrificed a creature (it was then exiled with the rest of their
    // graveyard), a Spirit token was created, and P0 gained 4 life.
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t
        .g
        .permanents()
        .any(|o| o.is_token() && o.chars.has_subtype("Spirit")));
    assert_eq!(t.life(P0), 24);
}
