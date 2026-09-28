//! Rulings batch S25 — casting a copy of a card (CR 707.12, 707.12a): the copy is cast
//! while the ability that makes it resolves, or not at all; a copy that isn't cast ceases
//! to exist (CR 707.10a, 704.5e).

use crate::r_s01_common::{supported, watch};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Isochron Scepter enters, exiling Lightning Bolt from P0's hand. Returns the
/// Scepter.
fn scepter_with_bolt(t: &mut TestGame) -> ObjectId {
    supported("Isochron Scepter");
    // "Imprint — When this artifact enters, you may exile an instant card with mana value
    // 2 or less from your hand. {2}, {T}: You may copy the exiled card. If you do, you may
    // cast the copy without paying its mana cost."
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    scepter
}

/// Copies of cards that still exist outside the stack.
fn card_copies_left(t: &TestGame) -> usize {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::CardCopy && t.g.is_live(o.id) && o.zone != Zone::Nowhere)
        .count()
}

#[test]
fn the_copy_is_cast_while_the_ability_resolves() {
    cr!("707.12", "608.2");
    ruling!(
        "Isochron Scepter",
        "You cast the copy while the ability is resolving and still on the stack. You can't wait to cast it later in the turn."
    );
    let mut t = TestGame::new(2);
    let scepter = scepter_with_bolt(&mut t);
    // When the copy's target is chosen, the Scepter's ability is still on the stack.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. }),
        |g| {
            g.stack
                .iter()
                .any(|id| g.obj(*id).kind == ObjKind::StackAbility)
        },
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, scepter, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(*seen.lock().unwrap(), vec![true]);
    // The copy is a spell on the stack now; the ability is gone.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.history.spells_cast.len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(card_copies_left(&t), 0);
}

#[test]
fn a_copy_you_dont_cast_ceases_to_exist() {
    cr!("707.12a", "707.10a", "704.5e");
    ruling!(
        "Isochron Scepter",
        "If you don't want to cast the copy, you can choose not to; the copy ceases to exist the next time state-based actions are checked."
    );
    let mut t = TestGame::new(2);
    let scepter = scepter_with_bolt(&mut t);
    // Copy it, but don't cast the copy.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.activate(P0, scepter, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert!(t.g.history.spells_cast.is_empty());
    assert_eq!(t.life(P1), 20);
    assert_eq!(card_copies_left(&t), 0);
    // Only the exiled card itself is left.
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Lightning Bolt").len(), 1);
    assert!(!t.in_hand(P0, "Lightning Bolt"));
}
