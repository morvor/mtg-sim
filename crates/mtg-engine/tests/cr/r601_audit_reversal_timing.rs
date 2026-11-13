//! CR 601.2 / 733 audit: a casting that can't be completed is reversed as a whole, with
//! nothing paid and nothing triggered (CR 601.2, 733.1), and a spell can begin to be cast
//! only when a rule or effect allows it (CR 601.3), alternative costs included.

use crate::r601_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const DASH: CastMethod = CastMethod::Keyword(KeywordKind::Dash);

#[test]
fn an_illegal_cast_is_reversed_without_cast_triggers_or_payments() {
    cr!("733.1", "601.2i", "601.2h");
    supported("Monastery Swiftspear");
    // Lightning Bolt with no red mana can't be paid: the Forest stays untapped and the
    // Swiftspear's prowess doesn't trigger. With a Mountain, the Bolt is cast and prowess
    // triggers.
    let mut t = TestGame::new(2);
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    t.lands(P0, "Forest", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(untapped_lands(&t, P0), 1);
    t.settle();
    assert_eq!(t.pt(spear), (1, 2), "no prowess trigger from a reversed cast");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    t.lands(P0, "Mountain", 1);
    let bolt = t.g.current(bolt);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(spear), (2, 3));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_spell_without_a_legal_target_cant_be_cast_and_nothing_is_paid() {
    cr!("601.2", "601.2c", "733.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    assert!(!can_cast(&mut t, P0, growth, CastMethod::Normal));
    assert!(t.cast(P0, growth).try_go().is_err());
    assert!(t.in_hand(P0, "Giant Growth"));
    assert_eq!(untapped_lands(&t, P0), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn an_alternative_cost_doesnt_change_when_a_spell_may_be_cast() {
    cr!("601.3", "702.109a");
    ruling!(
        "Mardu Scout",
        "You can cast a creature spell for its dash cost only when you otherwise could cast that creature spell."
    );
    // Mardu Scout's dash cost doesn't let it be cast during an opponent's turn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let scout = t.hand(P0, "Mardu Scout");
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, scout, DASH));
    assert!(!can_cast(&mut t, P0, scout, CastMethod::Normal));
    assert!(t.cast(P0, scout).method(DASH).try_go().is_err());
    assert!(t.in_hand(P0, "Mardu Scout"));
    // In its controller's main phase it may be.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, scout, DASH));
    t.cast(P0, scout).method(DASH).go();
    t.resolve_all();
    assert!(t.on_battlefield(scout));
}
