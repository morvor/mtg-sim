//! CR 603.11: static abilities linked to triggered abilities in the same paragraph.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::opening_hand::opening_hand_actions;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_static_ability_and_its_linked_trigger_share_one_paragraph() {
    cr!("603.11", "607.2h");
    // Watchful Naga: "You may exert this creature as it attacks. When you do, draw a card."
    let abs = abilities(&card("Watchful Naga"));
    assert_eq!(abs.len(), 2);
    assert!(matches!(&abs[0].kind, AbilityKind::Static(_)));
    assert!(matches!(&abs[1].kind, AbilityKind::Triggered(_)));
    assert_ne!(abs[0].link, 0);
    assert_eq!(abs[0].link, abs[1].link);

    // Exerting it triggers the linked ability.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Watchful Naga");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.attack(&[(n, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.obj(n).exerted);
    // It doesn't untap during its controller's next untap step.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj(n).tapped);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj(n).tapped);

    // Not exerting it: the triggered ability doesn't trigger.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Watchful Naga");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.attack(&[(n, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand);
    assert!(!t.obj(n).exerted);
}

#[test]
fn a_trigger_condition_may_be_written_after_the_effect() {
    cr!("603.11");
    // Sphinx of Foresight: "You may reveal this card from your opening hand. If you do,
    // scry 3 at the beginning of your first upkeep."
    let abs = abilities(&card("Sphinx of Foresight"));
    assert!(abs
        .iter()
        .all(|a| !matches!(a.kind, AbilityKind::Unsupported(_))));
    let mut t = TestGame::new(2);
    t.hand(P0, "Sphinx of Foresight");
    t.answer_yes(P0, true);
    opening_hand_actions(&mut t.g);
    // Not in the opponent's upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && format!("{d:?}").to_lowercase().contains("scry")));
}
