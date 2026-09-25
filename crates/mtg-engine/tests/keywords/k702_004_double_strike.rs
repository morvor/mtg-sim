//! CR 702.4 Double Strike.

use super::k702_001_010_common::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn double_striker_deals_damage_in_both_combat_damage_steps() {
    cr!("702.4a", "702.4b", "510.4");
    // Fencing Ace: 1/1 double strike.
    assert_eq!(printed_keywords("Fencing Ace", KeywordKind::DoubleStrike).len(), 1);
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Fencing Ace");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(ace, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 19);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn double_striker_and_its_blocker_in_the_second_step() {
    cr!("702.4b");
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Fencing Ace");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(ace, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, ace)]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(damage(&t, giant), 1);
    assert_eq!(damage(&t, ace), 0);
    go_to(&mut t, Step::EndOfCombat);
    // In the second step both the Ace (again) and the Giant (for the first time) deal
    // damage.
    assert_eq!(damage(&t, giant), 2);
    assert!(!t.on_battlefield(ace));
}

#[test]
fn a_double_striker_whose_blocker_died_deals_no_regular_damage_without_trample() {
    cr!("702.4b", "510.1c");
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Fencing Ace");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(ace, Entity::Player(P1))]);
    block(&mut t, P1, &[(goblin, ace)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(goblin));
    assert!(t.on_battlefield(ace));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn removing_double_strike_during_the_first_step_stops_regular_damage() {
    cr!("702.4c");
    ruling!(
        "Bonescythe Sliver",
        "If Bonescythe Sliver leaves the battlefield after other Sliver creatures you control have dealt first-strike damage but before regular combat damage, those Slivers won’t deal regular combat damage (unless they still have double strike for some other reason)."
    );
    let mut t = TestGame::new(2);
    let bonescythe = t.battlefield(P0, "Bonescythe Sliver");
    let metallic = t.battlefield(P0, "Metallic Sliver");
    assert!(t.obj_now(metallic).has_keyword(KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(metallic, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 19);
    t.g.destroy(bonescythe, None);
    t.settle();
    assert!(!t.obj_now(metallic).has_keyword(KeywordKind::DoubleStrike));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_creature_that_loses_double_strike_after_first_strike_damage_deals_no_normal_damage() {
    cr!("702.4c");
    ruling!(
        "Kwende, Pride of Femeref",
        "If a creature loses double strike after first strike damage is dealt, it won't deal normal combat damage."
    );
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Fencing Ace");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(ace, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    remove_kw(&mut t, ace, KeywordKind::DoubleStrike);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn giving_double_strike_to_a_first_striker_after_it_dealt_damage_lets_it_deal_again() {
    cr!("702.4d");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    t.lands(P0, "Mountain", 2);
    let rage = t.hand(P0, "Temur Battle Rage");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(knight, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    t.cast(P0, rage).target(knight).go();
    t.resolve();
    assert!(t.obj_now(knight).has_keyword(KeywordKind::DoubleStrike));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn gaining_double_strike_after_first_strike_damage_without_first_strike_deals_once() {
    cr!("702.4d", "702.4b");
    // A creature that had neither first strike nor double strike as the first step began
    // deals regular damage only, even if it gains double strike.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    grant(&mut t, bears, Keyword::new(KeywordKind::DoubleStrike));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn multiple_instances_of_double_strike_are_redundant() {
    cr!("702.4e");
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P0, "Fencing Ace");
    t.lands(P0, "Mountain", 2);
    let rage = t.hand(P0, "Temur Battle Rage");
    t.cast(P0, rage).target(ace).go();
    t.resolve();
    assert_eq!(instances(&t, ace, KeywordKind::DoubleStrike), 2);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(ace, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn hellbent_double_strike_is_lost_when_a_card_enters_the_hand_between_the_steps() {
    cr!("702.4c");
    ruling!(
        "Rakdos Pit Dragon",
        "If Rakdos Pit Dragon loses double strike after first strike combat damage has been dealt, it won't deal damage during the normal combat damage step."
    );
    assert_supported("Rakdos Pit Dragon");
    let mut t = TestGame::new(2);
    // Rakdos Pit Dragon: 3/3, "Hellbent — has double strike as long as you have no cards
    // in hand."
    let dragon = t.battlefield(P0, "Rakdos Pit Dragon");
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.obj_now(dragon).has_keyword(KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(dragon, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 17);
    t.g.draw_cards(P0, 1);
    t.g.recompute();
    assert!(!t.obj_now(dragon).has_keyword(KeywordKind::DoubleStrike));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn metalcraft_double_strike_is_checked_as_each_damage_step_begins() {
    cr!("702.4b", "702.4c");
    ruling!(
        "Auriok Edgewright",
        "As that second combat damage step begins, if Auriok Edgewright no longer has double strike (perhaps because an artifact creature you controlled was destroyed), Auriok Edgewright will not assign combat damage a second time."
    );
    assert_supported("Auriok Edgewright");
    let mut t = TestGame::new(2);
    let edgewright = t.battlefield(P0, "Auriok Edgewright");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let myr = t.battlefield(P0, "Memnite");
    assert!(t.obj_now(edgewright).has_keyword(KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(edgewright, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    t.g.destroy(myr, None);
    t.settle();
    assert!(!t.obj_now(edgewright).has_keyword(KeywordKind::DoubleStrike));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}
