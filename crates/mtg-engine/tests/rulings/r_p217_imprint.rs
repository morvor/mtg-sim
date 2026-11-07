//! Rulings batch P217 — imprint: cards exiled with a permanent (CR 607.2a), copied in
//! exile and cast without paying their mana cost (CR 707.12), or used for triggers.

use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Isochron Scepter
// ---------------------------------------------------------------------------

/// Isochron Scepter enters for P0, exiling Lightning Bolt from P0's hand. Returns (the
/// Scepter, the Bolt card in exile).
fn scepter_with_bolt(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Isochron Scepter");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    (scepter, bolt)
}

/// Activates the Scepter's "{2}, {T}: You may copy the exiled card. If you do, you may
/// cast the copy without paying its mana cost." (copying, casting, targeting P1); the
/// ability is left on the stack.
fn activate_scepter(t: &mut TestGame, scepter: ObjectId) {
    add_mana(t, P0, ManaType::C, 2);
    activate_containing(t, P0, scepter, "copy the exiled card").expect("activate");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
}

#[test]
fn the_scepter_leaving_doesnt_stop_the_copy_but_the_card_leaving_exile_does() {
    cr!("607.2a", "707.12", "113.7a", "400.7");
    ruling!(
        "Isochron Scepter",
        "If Isochron Scepter leaves the battlefield while the activated ability is on the stack, the ability can still make a copy. On the other hand, if the imprinted card leaves the exile zone while the activated ability is on the stack, the copy can't be made."
    );
    // The Scepter is destroyed in response: the Bolt is still copied and cast.
    let mut t = TestGame::new(2);
    let (scepter, _) = scepter_with_bolt(&mut t);
    activate_scepter(&mut t, scepter);
    destroy(&mut t, scepter);
    assert!(t.in_graveyard(P0, "Isochron Scepter"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.history.spells_cast.len(), 1);
    // The Bolt leaves exile in response: no copy.
    let mut t = TestGame::new(2);
    let (scepter, bolt) = scepter_with_bolt(&mut t);
    activate_scepter(&mut t, scepter);
    move_to(&mut t, bolt, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.g.history.spells_cast.is_empty());
}

// ---------------------------------------------------------------------------
// Mimic Vat
// ---------------------------------------------------------------------------

const VAT_TRIGGER: &str = "nontoken creature dies";

#[test]
fn mimic_vat_triggers_for_each_creature_but_keeps_at_most_one() {
    cr!("603.2c", "603.3b", "607.2a");
    ruling!(
        "Mimic Vat",
        "If multiple nontoken creatures are put into their owners' graveyards from the battlefield at the same time, the imprint ability will trigger that many times."
    );
    supported("Mimic Vat");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mimic Vat");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Wrath of God");
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    // Two triggers, which P0 orders; P0 exiles a card with each.
    assert_eq!(triggers_on_stack(&t, VAT_TRIGGER), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    // Only one card ends up exiled; the other is back in its owner's graveyard.
    let giant = t.in_exile("Hill Giant");
    let bears = t.in_exile("Grizzly Bears");
    assert!(giant ^ bears);
    assert_eq!(t.in_graveyard(P0, "Hill Giant"), !giant);
    assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), !bears);
}

#[test]
fn mimic_vat_triggers_for_a_creature_put_into_any_graveyard() {
    cr!("603.2", "700.4");
    ruling!(
        "Mimic Vat",
        "The imprint ability will trigger whenever a nontoken creature is put into any graveyard from the battlefield, not just your graveyard."
    );
    supported("Mimic Vat");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mimic Vat");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, VAT_TRIGGER), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // A token creature dying doesn't trigger it.
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    destroy(&mut t, token);
    assert_eq!(triggers_on_stack(&t, VAT_TRIGGER), 0);
}

// ---------------------------------------------------------------------------
// Panoptic Mirror
// ---------------------------------------------------------------------------

/// Activates Panoptic Mirror's "Imprint — {X}, {T}: You may exile an instant or sorcery
/// card with mana value X from your hand." with X = `x`, choosing `card`, and resolves
/// it; the Mirror is untapped again afterwards.
fn mirror_imprint(t: &mut TestGame, mirror: ObjectId, card: ObjectId, x: i64) {
    add_mana(t, P0, ManaType::C, x as u32);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    activate_containing(t, P0, mirror, "{X}").expect("activate");
    t.resolve();
    t.clear_answers();
    t.g.untap(mirror);
}

/// Goes to P0's next upkeep, with the Mirror's trigger on the stack.
fn to_p0_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

const MIRROR_TRIGGER: &str = "you may copy a card exiled";

#[test]
fn panoptic_mirror_copies_the_imprinted_card_in_exile_and_casts_it_for_free() {
    cr!("707.12", "607.2a", "118.9");
    ruling!(
        "Panoptic Mirror",
        "As Panoptic Mirror's triggered ability resolves, it allows you to create a copy of one of the instant or sorcery cards imprinted on Panoptic Mirror in the Exile zone (that's where the imprinted card is) and then cast it without paying its mana cost."
    );
    supported("Panoptic Mirror");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Panoptic Mirror");
    let bolt = t.hand(P0, "Lightning Bolt");
    mirror_imprint(&mut t, mirror, bolt, 1);
    assert!(t.in_exile("Lightning Bolt"));
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, MIRROR_TRIGGER), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // The copy was cast (P0 has no mana) and dealt 3 damage; the card stays exiled.
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn panoptic_mirror_triggers_once_per_upkeep_and_does_nothing_with_no_card() {
    cr!("603.2", "607.2a", "608.2b");
    ruling!(
        "Panoptic Mirror",
        "The triggered ability triggers only once each upkeep, not once per imprinted card. If no cards are imprinted on Panoptic Mirror when the triggered ability resolves, it does nothing."
    );
    supported("Panoptic Mirror");
    // Two imprinted cards: one trigger; P0 copies one of them.
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Panoptic Mirror");
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    mirror_imprint(&mut t, mirror, bolt, 1);
    mirror_imprint(&mut t, mirror, shock, 1);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Shock"));
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, MIRROR_TRIGGER), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(t.g.current(bolt))]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.life(P1), 17);
    // No imprinted card: it does nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panoptic Mirror");
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, MIRROR_TRIGGER), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.g.history.spells_cast.is_empty());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn panoptic_mirror_imprints_a_split_card_by_its_combined_mana_value_and_casts_one_half() {
    cr!("709.4", "709.3", "707.12");
    ruling!(
        "Panoptic Mirror",
        "To imprint a split card, you pay X equal to card's mana value, determined by the combined mana cost of its two halves. If the copied card is a split card, you choose which one side of it to cast, but you can't cast both sides."
    );
    supported("Panoptic Mirror");
    supported("Fire // Ice");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Panoptic Mirror");
    let fire_ice = t.hand(P0, "Fire // Ice");
    // X = 2 (one half's mana value) can't exile it.
    mirror_imprint(&mut t, mirror, fire_ice, 2);
    assert_eq!(t.zone(fire_ice), Zone::Hand(P0));
    // X = 4 (the combined mana value) can.
    mirror_imprint(&mut t, mirror, fire_ice, 4);
    assert!(t.in_exile("Fire // Ice"));
    // P0 copies it and casts the copy as Ice ("Tap target permanent. Draw a card.").
    let giant = t.battlefield(P1, "Hill Giant");
    to_p0_upkeep(&mut t);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    let ways: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose what to cast" => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ways, vec![vec!["Cast Fire".to_string(), "Cast Ice".to_string()]]);
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert!(t.obj_now(giant).tapped);
    // Fire wasn't cast too.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.g.obj(giant).damage, 0);
}

#[test]
fn a_card_imprinted_in_response_to_the_upkeep_trigger_can_be_copied() {
    cr!("603.3", "607.2a", "117.3a");
    ruling!(
        "Panoptic Mirror",
        "You may imprint a card on Panoptic Mirror in response to the upkeep-triggered ability. If you do, that card is available to copy when the triggered ability resolves."
    );
    supported("Panoptic Mirror");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Panoptic Mirror");
    let bolt = t.hand(P0, "Lightning Bolt");
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, MIRROR_TRIGGER), 1);
    // In response, P0 imprints Lightning Bolt (resolving just that activation).
    mirror_imprint(&mut t, mirror, bolt, 1);
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(triggers_on_stack(&t, MIRROR_TRIGGER), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------
// Spellbinder
// ---------------------------------------------------------------------------

/// Spellbinder enters for P0, exiling `instant` from P0's hand.
fn spellbinder_with(t: &mut TestGame, instant: &str) -> ObjectId {
    supported("Spellbinder");
    let card = t.hand(P0, instant);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let sb = t.enter(P0, "Spellbinder");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    t.clear_answers();
    sb
}

#[test]
fn spellbinder_imprints_only_as_it_enters_not_when_attached() {
    cr!("603.6a", "702.6a", "301.5c");
    ruling!(
        "Spellbinder",
        "Spellbinder imprints a card only once, when it enters. Attaching Spellbinder to a creature doesn't trigger its \"enters\" ability."
    );
    let mut t = TestGame::new(2);
    let sb = spellbinder_with(&mut t, "Lightning Bolt");
    t.hand(P0, "Shock");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Equip {4}.
    add_mana(&mut t, P0, ManaType::C, 4);
    activate_containing(&mut t, P0, sb, "Equip").expect("equip");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(sb).attached_to, Some(Entity::Object(bears)));
    // No "enters" trigger; Shock stays in hand.
    assert_eq!(triggers_on_stack(&t, "enters"), 0);
    assert!(t.in_hand(P0, "Shock"));
    assert!(!t.in_exile("Shock"));
}

#[test]
fn spellbinder_copies_the_imprinted_instant_in_exile_and_casts_it_for_free() {
    cr!("707.12", "607.2a", "118.9");
    ruling!(
        "Spellbinder",
        "Spellbinder's second ability allows you to create a copy of the imprinted instant card in the Exile zone (that's where the imprinted instant card is) and then cast it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    let sb = spellbinder_with(&mut t, "Lightning Bolt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(sb, Entity::Object(bears)));
    t.g.recompute();
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // P0 has no mana: the copy was cast for free. 2 combat damage + 3.
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.life(P1), 15);
    assert!(t.in_exile("Lightning Bolt"));
}
