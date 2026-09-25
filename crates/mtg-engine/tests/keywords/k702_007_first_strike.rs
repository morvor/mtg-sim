//! CR 702.7 First Strike.

use super::k702_001_010_common::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn first_striker_kills_its_blocker_before_it_deals_damage() {
    cr!("702.7a", "702.7b", "510.4");
    let mut t = TestGame::new(2);
    // Youthful Knight: 2/1 first strike.
    assert_eq!(printed_keywords("Youthful Knight", KeywordKind::FirstStrike).len(), 1);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(knight, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, knight)]);
    go_to(&mut t, Step::FirstStrikeDamage);
    // Only the first striker has dealt damage so far.
    assert_eq!(damage(&t, bears), 2);
    assert_eq!(damage(&t, knight), 0);
    t.settle();
    assert!(!t.on_battlefield(bears));
    go_to(&mut t, Step::EndOfCombat);
    assert!(step_happened(&t, Step::FirstStrikeDamage));
    assert!(step_happened(&t, Step::CombatDamage));
    assert!(t.on_battlefield(knight));
    assert_eq!(damage(&t, knight), 0);
}

#[test]
fn without_first_or_double_strike_there_is_one_combat_damage_step() {
    cr!("702.7b", "510.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!step_happened(&t, Step::FirstStrikeDamage));
    assert!(step_happened(&t, Step::CombatDamage));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn creatures_without_first_strike_deal_damage_in_the_second_step() {
    cr!("702.7b");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18, "only the first striker");
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 16, "then the rest; the first striker doesn't deal again");
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_blocking_first_striker_also_strikes_first() {
    cr!("702.7b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let knight = t.battlefield(P1, "Youthful Knight");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(knight, bears)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(knight));
}

#[test]
fn gaining_first_strike_after_first_strike_damage_still_deals_regular_damage() {
    cr!("702.7c");
    ruling!(
        "Kindled Fury",
        "Giving a creature first strike after creatures with first strike deal combat damage doesn't prevent that creature from dealing combat damage."
    );
    ruling!(
        "Seize the Initiative",
        "A creature that gains first strike after other creatures with first strike or double strike deal combat damage in the first combat damage step will still deal damage in the second combat damage step."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let fury = t.hand(P0, "Kindled Fury");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    t.cast(P0, fury).target(bears).go();
    t.resolve();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    go_to(&mut t, Step::EndOfCombat);
    // The Bears (3/2 now) deal regular combat damage.
    assert_eq!(t.life(P1), 15);
}

#[test]
fn losing_first_strike_after_first_strike_damage_doesnt_allow_regular_damage() {
    cr!("702.7c");
    ruling!(
        "Striking Sliver",
        "If Striking Sliver leaves the battlefield after Sliver creatures you control have dealt first-strike damage but before regular combat damage, those Slivers won’t deal regular combat damage (unless they have double strike for some reason)."
    );
    let mut t = TestGame::new(2);
    let striking = t.battlefield(P0, "Striking Sliver");
    let metallic = t.battlefield(P0, "Metallic Sliver");
    assert!(t.obj_now(metallic).has_keyword(KeywordKind::FirstStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(metallic, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 19);
    t.g.destroy(striking, None);
    t.settle();
    assert!(!t.obj_now(metallic).has_keyword(KeywordKind::FirstStrike));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19, "no regular combat damage");
}

#[test]
fn losing_first_strike_but_having_double_strike_deals_regular_damage() {
    cr!("702.7c", "702.4b");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    grant(&mut t, knight, Keyword::new(KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(knight, Entity::Player(P1))]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    remove_kw(&mut t, knight, KeywordKind::FirstStrike);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn multiple_instances_of_first_strike_are_redundant() {
    cr!("702.7d");
    ruling!(
        "Striking Sliver",
        "for some abilities, like flying, having more than one instance of the ability doesn’t provide any additional benefit"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Striking Sliver");
    t.battlefield(P0, "Striking Sliver");
    let metallic = t.battlefield(P0, "Metallic Sliver");
    // Two Striking Slivers: two instances of first strike.
    assert_eq!(instances(&t, metallic, KeywordKind::FirstStrike), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    grant(&mut t, bears, Keyword::new(KeywordKind::FirstStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(metallic, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, metallic)]);
    go_to(&mut t, Step::EndOfCombat);
    // Still just first strike: it deals damage at the same time as another first
    // striker, once.
    assert!(!t.on_battlefield(metallic));
    assert_eq!(damage(&t, bears), 1);
}

#[test]
fn first_strike_and_double_strike_together_deal_damage_twice_not_three_times() {
    cr!("702.7d", "702.4b");
    ruling!(
        "Kwende, Pride of Femeref",
        "A creature with first strike and double strike deals combat damage the same as a creature with double strike. It doesn't deal damage three times or before other creatures with first strike."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kwende, Pride of Femeref");
    let knight = t.battlefield(P0, "Youthful Knight");
    assert!(t.obj_now(knight).has_keyword(KeywordKind::FirstStrike));
    assert!(t.obj_now(knight).has_keyword(KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(knight, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn first_strike_only_while_attacking() {
    cr!("702.7a", "702.7b");
    assert_supported("Kor Scythemaster");
    // Kor Scythemaster: 3/1, "This creature has first strike as long as it's attacking."
    let mut t = TestGame::new(2);
    let kor = t.battlefield(P0, "Kor Scythemaster");
    assert!(!t.obj_now(kor).has_keyword(KeywordKind::FirstStrike));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(kor, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, kor)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(kor));
    assert!(!t.on_battlefield(bears));

    // Blocking, it doesn't have first strike: the two trade.
    let mut t = TestGame::new(2);
    let kor = t.battlefield(P1, "Kor Scythemaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(kor, bears)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!step_happened(&t, Step::FirstStrikeDamage));
    assert!(!t.on_battlefield(kor));
    assert!(!t.on_battlefield(bears));
}
