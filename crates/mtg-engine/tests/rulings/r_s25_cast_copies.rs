//! Rulings batch S25 — casting a copy of a card (CR 707.12, 707.12a): the copy is cast
//! while the ability that makes it resolves, or not at all; a copy that isn't cast ceases
//! to exist (CR 707.10a, 704.5e).

use crate::r_s01_common::{attack_with, place, supported, watch};
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

/// P0's Narset, Enlightened Exile (a 3/4: "Whenever Narset attacks, exile target
/// noncreature, nonland card with mana value less than Narset's power from a graveyard and
/// copy it. You may cast the copy without paying its mana cost.") attacks, targeting the
/// card `name` in P1's graveyard. Returns (Narset, the card).
fn narset_attacks_targeting(t: &mut TestGame, name: &str) -> (ObjectId, ObjectId) {
    supported("Narset, Enlightened Exile");
    let narset = t.battlefield(P0, "Narset, Enlightened Exile");
    let card = place(t, P1, name, Zone::Graveyard(P1));
    t.answer_targets(P0, &[Entity::Object(card)]);
    attack_with(t, &[(narset, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    (narset, card)
}

#[test]
fn narset_s_copy_you_dont_cast_ceases_to_exist() {
    cr!("707.12a", "704.5e");
    ruling!(
        "Narset, Enlightened Exile",
        "If you don't want to cast the copy, you can choose not to; the copy ceases to exist the next time state-based actions are performed."
    );
    let mut t = TestGame::new(2);
    let (_, bolt) = narset_attacks_targeting(&mut t, "Lightning Bolt");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.g.stack.is_empty());
    assert!(t.g.history.spells_cast.is_empty());
    assert_eq!(t.life(P1), 20);
    assert_eq!(card_copies_left(&t), 0);
}

#[test]
fn narset_casts_a_copy_of_the_exiled_card() {
    cr!("707.12", "608.2");
    ruling!(
        "Narset, Enlightened Exile",
        "You cast the copy while the ability is resolving and still on the stack."
    );
    let mut t = TestGame::new(2);
    let (narset, bolt) = narset_attacks_targeting(&mut t, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    // The copy is a spell on the stack (with the prowess trigger it caused); the card
    // stays in exile.
    assert!(t
        .g
        .stack
        .iter()
        .any(|id| t.g.obj(*id).kind == ObjKind::CardCopy));
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(narset), (4, 5));
    assert_eq!(card_copies_left(&t), 0);
    assert_eq!(t.zone(bolt), Zone::Exile);
}

#[test]
fn narset_s_target_must_have_mana_value_less_than_its_power_as_it_resolves() {
    cr!("608.2b", "115.1");
    ruling!(
        "Narset, Enlightened Exile",
        "If another effect causes Narset's power to be less than or equal to the mana value of the target card as the ability tries to resolve, the target is illegal."
    );
    // Doom Blade has mana value 2, less than Narset's power 3 as the trigger is put on the
    // stack; Disfigure ("Target creature gets -2/-2 until end of turn.") then makes Narset
    // a 1/2.
    let mut t = TestGame::new(2);
    let (narset, blade) = narset_attacks_targeting(&mut t, "Doom Blade");
    let disfigure = t.hand(P1, "Disfigure");
    t.lands(P1, "Swamp", 1);
    t.cast_with(P1, disfigure, &[Entity::Object(narset)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(narset), (1, 2));
    t.resolve_all();
    assert_eq!(t.zone(blade), Zone::Graveyard(P1));
    assert!(t.g.history.spells_cast.iter().all(|(p, _)| *p != P0));
    assert_eq!(card_copies_left(&t), 0);
}

#[test]
fn shiko_s_copy_you_dont_cast_ceases_to_exist() {
    cr!("707.12a", "704.5e");
    ruling!(
        "Shiko, Paragon of the Way",
        "If you don’t want to cast the copy, you can choose not to; the copy ceases to exist the next time state-based actions are checked."
    );
    supported("Shiko, Paragon of the Way");
    // "When Shiko enters, exile target nonland card with mana value 3 or less from your
    // graveyard. Copy it, then you may cast the copy without paying its mana cost."
    let mut t = TestGame::new(2);
    let divination = place(&mut t, P0, "Divination", Zone::Graveyard(P0));
    t.answer_targets(P0, &[Entity::Object(divination)]);
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.enter(P0, "Shiko, Paragon of the Way");
    t.resolve_all();
    assert_eq!(t.zone(divination), Zone::Exile);
    assert!(t.g.history.spells_cast.is_empty());
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(card_copies_left(&t), 0);
}
