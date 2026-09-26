//! CR 702.123 Fabricate.

use crate::common_k702_111_124::*;
use crate::k702_001_010_common::custom_card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn fabricate_puts_counters_on_it() {
    cr!("702.123", "702.123a");
    ruling!(
        "Weaponcraft Enthusiast",
        "Fabricate doesn't cause the creature with the ability to enter the battlefield with +1/+1 counters already on it."
    );
    assert_supported_card("Weaponcraft Enthusiast");
    let mut t = TestGame::new(2);
    // Weaponcraft Enthusiast: 0/1, fabricate 2.
    t.lands(P0, "Swamp", 3);
    let c = t.hand(P0, "Weaponcraft Enthusiast");
    t.cast(P0, c).go();
    t.resolve();
    // It entered as a 0/1; fabricate is on the stack.
    assert!(t.on_battlefield(c));
    assert_eq!(t.pt(c), (0, 1));
    assert_eq!(on_stack(&t, "Fabricate 2"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert_eq!(t.pt(c), (2, 3));
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn if_you_dont_put_counters_you_create_servos() {
    cr!("702.123a");
    let mut t = TestGame::new(2);
    let c = t.enter(P0, "Weaponcraft Enthusiast");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(c, counters::PLUS1), 0);
    let servos = tokens_of(&t, P0);
    assert_eq!(servos.len(), 2);
    for s in servos {
        let o = t.g.obj(s);
        assert!(o.chars.colors.is_colorless());
        assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
        assert!(o.chars.has_subtype("Servo"));
        assert_eq!((o.power(), o.toughness()), (1, 1));
        assert_eq!(o.controller, P0);
    }
}

#[test]
fn if_it_left_the_battlefield_you_create_servos() {
    cr!("702.123a");
    ruling!(
        "Weaponcraft Enthusiast",
        "If you can't put +1/+1 counters on the creature for any reason as fabricate resolves (for instance, if it's no longer on the battlefield), you just create Servo tokens."
    );
    let mut t = TestGame::new(2);
    let c = t.enter(P0, "Weaponcraft Enthusiast");
    t.settle();
    // In response, it's destroyed.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(c).go();
    t.resolve();
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    // The controller isn't asked; the tokens are created.
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 2);
}

#[test]
fn each_instance_of_fabricate_triggers_separately() {
    cr!("702.123b");
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Twice Fabricated",
        "Artifact Creature — Construct",
        "{4}",
        Some((1, 1)),
        "Fabricate 1\nFabricate 2",
    );
    let id = t.g.create_card_object(std::sync::Arc::new(def), P0, Zone::Nowhere);
    let id = t
        .g
        .move_object(id, Zone::Battlefield, events::MoveCause::Effect, Some(P0))
        .unwrap();
    t.settle();
    assert_eq!(on_stack(&t, "Fabricate 1"), 1);
    assert_eq!(on_stack(&t, "Fabricate 2"), 1);
    // Counters for one, Servos for the other.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let counters_on = t.counters(id, counters::PLUS1);
    let servos = tokens_of(&t, P0).len() as u32;
    assert!(
        (counters_on, servos) == (2, 1) || (counters_on, servos) == (1, 2),
        "{counters_on} counters, {servos} servos"
    );
}
