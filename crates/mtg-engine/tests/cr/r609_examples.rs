//! The worked examples of CR 609 (effects), played out as tests. The CR 609.2 and 609.3
//! examples are cited by tests in `r609_609_effects.rs`.

use crate::r600_examples_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn vedalken_orrery_and_shamans_trance_let_a_flashback_sorcery_be_cast_from_another_graveyard() {
    cr!("609.4a", "609.4", "702.34a");
    cr_example!("609.4a", "Vedalken Orrery");
    supported("Vedalken Orrery");
    supported("Shaman's Trance");
    supported("Firebolt");
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    // Firebolt: a sorcery with flashback {4}{R}, in the other player's graveyard.
    let firebolt = t.graveyard(P1, "Firebolt");
    let own = t.graveyard(P1, "Firebolt");
    t.lands(P0, "Mountain", 8);
    t.lands(P1, "Mountain", 5);
    // During the other player's main phase.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, firebolt, &flashback));
    assert!(can_cast(&mut t, P1, own, &flashback));
    let trance = t.hand(P0, "Shaman's Trance");
    t.cast(P0, trance).go();
    t.resolve();
    // The other player can't cast spells from their own graveyard this turn.
    assert!(!can_cast(&mut t, P1, own, &flashback));
    // P0 may: as though it were in P0's graveyard, and as though it had flash.
    assert!(can_cast(&mut t, P0, firebolt, &flashback));
    t.cast(P0, firebolt).method(flashback.clone()).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Firebolt"));
}
