//! CR 702.107 Outlast.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::{activate_named, tokens};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn outlast_puts_a_counter_on_the_creature_for_its_cost_and_tapping() {
    cr!("702.107", "702.107a");
    ruling!(
        "Ainok Bond-Kin",
        "These counters could come from an outlast ability, but any +1/+1 counter on the creature will count."
    );
    assert_supported("Ainok Bond-Kin");
    let mut t = TestGame::new(2);
    // Ainok Bond-Kin: 2/1, "Outlast {1}{W}", "Each creature you control with a +1/+1
    // counter on it has first strike."
    let ainok = t.battlefield(P0, "Ainok Bond-Kin");
    let lands = t.lands(P0, "Plains", 2);
    assert!(!t.obj_now(ainok).chars.has_keyword(KeywordKind::FirstStrike));
    activate_named(&mut t, P0, ainok, "Outlast", 0).unwrap();
    assert!(t.obj_now(ainok).tapped);
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    t.resolve_all();
    assert_eq!(t.counters(ainok, counters::PLUS1), 1);
    assert_eq!(t.pt(ainok), (3, 2));
    assert!(t.obj_now(ainok).chars.has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn outlast_can_be_activated_only_as_a_sorcery() {
    cr!("702.107a");
    let mut t = TestGame::new(2);
    let ainok = t.battlefield(P0, "Ainok Bond-Kin");
    t.lands(P0, "Plains", 4);
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(activate_named(&mut t, P0, ainok, "Outlast", 0).is_err());
    // Not while the stack isn't empty.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(activate_named(&mut t, P0, ainok, "Outlast", 0).is_err());
    t.resolve_all();
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(activate_named(&mut t, P0, ainok, "Outlast", 0).is_err());
    // In its controller's main phase with an empty stack, it can.
    t.set_step(P0, Step::PostcombatMain);
    assert!(activate_named(&mut t, P0, ainok, "Outlast", 0).is_ok());
}

#[test]
fn outlast_needs_the_creature_to_have_been_controlled_since_the_turn_began() {
    cr!("702.107a");
    ruling!(
        "Ainok Bond-Kin",
        "A creature's outlast ability can't be activated unless that creature has been under your control continuously since the beginning of your turn."
    );
    let mut t = TestGame::new(2);
    let ainok = t.battlefield_sick(P0, "Ainok Bond-Kin");
    t.lands(P0, "Plains", 2);
    assert!(activate_named(&mut t, P0, ainok, "Outlast", 0).is_err());
    assert_eq!(t.counters(ainok, counters::PLUS1), 0);
    // Tapped, it can't be activated either.
    let other = t.battlefield(P0, "Ainok Bond-Kin");
    t.g.objects[other.0 as usize].tapped = true;
    assert!(activate_named(&mut t, P0, other, "Outlast", 0).is_err());
}

#[test]
fn activating_outlast_can_trigger_an_ability() {
    cr!("702.107a");
    ruling!(
        "Herald of Anafenza",
        "The Warrior token is created before the +1/+1 counter is put on Herald of Anafenza."
    );
    assert_supported("Herald of Anafenza");
    let mut t = TestGame::new(2);
    // Herald of Anafenza: "Whenever you activate this creature's outlast ability, create
    // a 1/1 white Warrior creature token."
    let herald = t.battlefield(P0, "Herald of Anafenza");
    t.lands(P0, "Plains", 3);
    activate_named(&mut t, P0, herald, "Outlast", 0).unwrap();
    t.settle();
    // The trigger is above the outlast ability: the token comes first.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(tokens(&t, P0), 1);
    assert_eq!(t.counters(herald, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.counters(herald, counters::PLUS1), 1);
    assert_eq!(tokens(&t, P0), 1);
}

#[test]
fn a_granted_outlast_works_like_a_printed_one() {
    cr!("702.107a");
    assert_supported("Arcus Acolyte");
    let mut t = TestGame::new(2);
    // Arcus Acolyte: "Each other creature you control without a +1/+1 counter on it has
    // outlast {G/W}."
    t.battlefield(P0, "Arcus Acolyte");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Outlast));
    assert!(!t.obj_now(theirs).chars.has_keyword(KeywordKind::Outlast));
    t.lands(P0, "Forest", 1);
    activate_named(&mut t, P0, bears, "Outlast", 0).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // With a +1/+1 counter on it, it no longer has outlast.
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Outlast));
}
