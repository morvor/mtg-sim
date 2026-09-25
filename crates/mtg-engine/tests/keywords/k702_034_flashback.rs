//! CR 702.34 Flashback.

use crate::common_k702_011_017::*;
use crate::common_k702_027_037::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

#[test]
fn flashback_casts_the_card_from_the_graveyard_then_exiles_it() {
    cr!("702.34", "702.34a");
    ruling!(
        "Think Twice",
        "You can cast a spell using flashback even if it was somehow put into your graveyard without having been cast."
    );
    assert_supported("Think Twice");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let tt = t.graveyard(P0, "Think Twice");
    let hand = t.hand_size(P0);
    assert!(can_cast(&mut t, P0, tt, FLASHBACK));
    // Not with its mana cost, and not normally.
    assert!(!can_cast(&mut t, P0, tt, CastMethod::Normal));
    t.cast(P0, tt).method(FLASHBACK).go();
    // The flashback cost {2}{U} was paid rather than {1}{U}.
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Think Twice"));
    assert!(!t.in_graveyard(P0, "Think Twice"));
}

#[test]
fn a_card_cast_normally_isnt_exiled_and_only_flashback_allows_graveyard_casting() {
    cr!("702.34a");
    assert_supported("Divination");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    // A card without flashback can't be cast from the graveyard.
    let div = t.graveyard(P0, "Divination");
    assert!(!can_cast(&mut t, P0, div, CastMethod::Normal));
    // Cast from the hand, Think Twice goes to the graveyard, and can then be flashed back.
    let tt = t.hand(P0, "Think Twice");
    t.cast(P0, tt).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Think Twice"));
    let tt = t.g.find_in_zone(Zone::Graveyard(P0), "Think Twice")[0];
    t.cast(P0, tt).method(FLASHBACK).go();
    t.resolve();
    assert!(t.in_exile("Think Twice"));
}

#[test]
fn a_flashback_spell_is_exiled_however_it_leaves_the_stack() {
    cr!("702.34a");
    ruling!(
        "Think Twice",
        "A spell cast using flashback will always be exiled afterward, whether it resolves, is countered, or leaves the stack in some other way."
    );
    ruling!(
        "Remand",
        "If you target a card that was cast with flashback with Remand, the card will still be exiled."
    );
    assert_supported("Counterspell");
    assert_supported("Unsubstantiate");
    // Countered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    let tt = t.graveyard(P0, "Think Twice");
    let spell = t.cast(P0, tt).method(FLASHBACK).go();
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    assert!(!t.in_graveyard(P0, "Think Twice"));
    // Returned to its owner's hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    let tt = t.graveyard(P0, "Think Twice");
    let spell = t.cast(P0, tt).method(FLASHBACK).go();
    let un = t.hand(P1, "Unsubstantiate");
    t.cast(P1, un).target(spell).go();
    t.resolve();
    assert!(t.in_exile("Think Twice"));
    assert!(!t.in_hand(P0, "Think Twice"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn flashback_follows_the_timing_rules_of_the_cards_type() {
    cr!("702.34a");
    ruling!(
        "Faithless Looting",
        "For instance, you can cast a sorcery using flashback only when you could normally cast a sorcery."
    );
    assert_supported("Faithless Looting");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let fl = t.graveyard(P0, "Faithless Looting");
    assert!(can_cast(&mut t, P0, fl, FLASHBACK));
    t.set_step(P0, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, fl, FLASHBACK));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, fl, FLASHBACK));
    // An instant can be flashed back at instant speed.
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    assert!(can_cast(&mut t, P0, tt, FLASHBACK));
}

#[test]
fn a_flashback_cost_can_include_other_costs_and_is_modified_like_any_cost() {
    cr!("702.34a");
    ruling!(
        "Deep Analysis",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a flashback cost) you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell is determined only by its mana cost"
    );
    assert_supported("Deep Analysis");
    assert_supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 3);
    let da = t.graveyard(P0, "Deep Analysis");
    let spell = t.cast(P0, da).method(FLASHBACK).target(P0).go();
    // {1}{U} + {1} from Thalia, and 3 life.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.obj_now(spell).chars.mana_value(), 4);
    t.resolve();
    assert!(t.in_exile("Deep Analysis"));
}

#[test]
fn only_an_instant_or_sorcery_spell_can_be_cast_with_flashback() {
    cr!("702.34a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let def = custom_card(
        "Flashback Golem",
        "Artifact Creature — Golem",
        Some((2, 2)),
        "Flashback {1}",
    );
    let golem = t.custom(P0, def, Zone::Graveyard(P0));
    assert!(t.obj_now(golem).has_keyword(KeywordKind::Flashback));
    assert!(!can_cast(&mut t, P0, golem, FLASHBACK));
    assert!(t.cast(P0, golem).method(FLASHBACK).try_go().is_err());
}

#[test]
fn a_granted_flashback_ability_can_cost_the_cards_mana_cost() {
    cr!("702.34a", "400.7g");
    assert_supported("Snapcaster Mage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    assert!(!can_cast(&mut t, P0, bolt, FLASHBACK));
    let mage = t.hand(P0, "Snapcaster Mage");
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.cast(P0, mage).go();
    t.resolve_all();
    assert!(t.obj_now(bolt).has_keyword(KeywordKind::Flashback));
    t.cast(P0, bolt).method(FLASHBACK).target(P1).go();
    // Its mana cost {R}.
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
}
