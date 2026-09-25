//! CR 702.41 Affinity.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is_land() && !o.tapped)
        .count()
}

#[test]
fn affinity_reduces_the_cost_by_one_for_each_matching_permanent() {
    cr!("702.41", "702.41a");
    assert_supported("Frogmite");
    let mut t = TestGame::new(2);
    // Frogmite {4}, affinity for artifacts: with two artifacts it costs {2}.
    t.battlefield(P0, "Memnite");
    t.battlefield(P0, "Memnite");
    t.lands(P0, "Wastes", 3);
    let frog = t.hand(P0, "Frogmite");
    t.cast(P0, frog).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Frogmite").len(), 1);
    // With four artifacts (the first Frogmite, two Memnites, and another) it's free.
    t.battlefield(P0, "Memnite");
    let frog2 = t.hand(P0, "Frogmite");
    t.cast(P0, frog2).go();
    assert_eq!(untapped_lands(&t, P0), 1);
}

#[test]
fn only_permanents_you_control_count() {
    cr!("702.41a");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Memnite");
    t.battlefield(P1, "Memnite");
    t.battlefield(P0, "Memnite");
    t.lands(P0, "Wastes", 2);
    let frog = t.hand(P0, "Frogmite");
    // Costs {3}: only one artifact is P0's.
    assert!(t.cast(P0, frog).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    t.cast(P0, frog).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn affinity_reduces_only_generic_mana() {
    cr!("702.41a");
    assert_supported("Thoughtcast");
    let mut t = TestGame::new(2);
    // Thoughtcast {4}{U} with six artifacts still costs {U}.
    for _ in 0..6 {
        t.battlefield(P0, "Memnite");
    }
    t.lands(P0, "Wastes", 2);
    let tc = t.hand(P0, "Thoughtcast");
    assert!(t.cast(P0, tc).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Island", 1);
    t.cast(P0, tc).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn the_cost_is_locked_in_before_it_is_paid() {
    cr!("702.41a");
    // Paying with Lotus Petal sacrifices an artifact, but the total cost was already
    // determined with it counted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lotus Petal");
    t.battlefield(P0, "Memnite");
    t.lands(P0, "Wastes", 1);
    let frog = t.hand(P0, "Frogmite");
    // {4} - 2 = {2}: Wastes and Lotus Petal.
    t.cast(P0, frog).go();
    assert!(t.in_graveyard(P0, "Lotus Petal"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Frogmite").len(), 1);
}

#[test]
fn affinity_for_outlaws() {
    cr!("702.41a");
    ruling!(
        "Hellspur Brute",
        "A card, spell, or permanent is an outlaw if it has the Assassin, Mercenary, Pirate, Rogue, or Warlock creature type."
    );
    assert_supported("Hellspur Brute");
    let mut t = TestGame::new(2);
    // Hellspur Brute {4}{R}; two Pirates and a non-outlaw: {2}{R}.
    t.battlefield(P0, "Kitesail Freebooter");
    t.battlefield(P0, "Kitesail Freebooter");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let brute = t.hand(P0, "Hellspur Brute");
    t.cast(P0, brute).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn each_instance_of_affinity_applies() {
    cr!("702.41b");
    let def = with_cost(
        custom_card(
            "Doubly Affine Golem",
            "Artifact Creature — Golem",
            Some((4, 4)),
            "Affinity for artifacts\nAffinity for artifacts",
        ),
        "{6}",
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Memnite");
    }
    let g = t.custom(P0, def, Zone::Hand(P0));
    // {6} - 3 - 3 = {0}.
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Doubly Affine Golem").len(), 1);
}
