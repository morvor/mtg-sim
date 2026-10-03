//! CR 702.27 Buyback.

use crate::common_k702_011_017::*;
use crate::common_k702_027_037::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn paying_buyback_returns_the_spell_to_its_owners_hand() {
    cr!("702.27", "702.27a");
    assert_supported("Whispers of the Muse");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let w = t.hand(P0, "Whispers of the Muse");
    let hand = t.hand_size(P0);
    t.cast(P0, w).kicked(true).go();
    assert_eq!(optional_costs_offered(&t, P0), vec!["buyback".to_string()]);
    // {U} plus the buyback cost {5}.
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    // It drew a card and came back: one more card in hand than before casting.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_hand(P0, "Whispers of the Muse"));
    assert!(!t.in_graveyard(P0, "Whispers of the Muse"));
}

#[test]
fn without_buyback_the_spell_goes_to_the_graveyard() {
    cr!("702.27a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let w = t.hand(P0, "Whispers of the Muse");
    t.cast(P0, w).kicked(false).go();
    assert_eq!(untapped_lands(&t, P0), 5);
    t.resolve();
    assert!(t.in_graveyard(P0, "Whispers of the Muse"));
    assert!(!t.in_hand(P0, "Whispers of the Muse"));
}

#[test]
fn buyback_is_an_additional_cost_added_to_the_total_cost() {
    cr!("702.27a", "601.2b", "601.2f", "601.2h");
    let mut t = TestGame::new(2);
    // {U} + {5} buyback can't be paid with five lands: announcing the buyback makes the
    // total cost unpayable, so the casting is illegal and undone.
    t.lands(P0, "Island", 5);
    let w = t.hand(P0, "Whispers of the Muse");
    assert!(t.cast(P0, w).kicked(true).try_go().is_err());
    assert!(t.in_hand(P0, "Whispers of the Muse"));
    assert_eq!(untapped_lands(&t, P0), 5);
    // Without the buyback it costs only its mana cost.
    t.clear_answers();
    let spell = t.cast(P0, w).kicked(false).go();
    assert_eq!(untapped_lands(&t, P0), 4);
    // The mana value of the spell is still that of its mana cost.
    assert_eq!(t.obj_now(spell).chars.mana_value(), 1);
    t.resolve();
    assert!(t.in_graveyard(P0, "Whispers of the Muse"));
}

#[test]
fn a_countered_buyback_spell_goes_to_the_graveyard() {
    cr!("702.27a");
    assert_supported("Counterspell");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.lands(P1, "Island", 2);
    let w = t.hand(P0, "Whispers of the Muse");
    let spell = t.cast(P0, w).kicked(true).go();
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    // It never resolved, so the buyback replacement didn't apply.
    assert!(t.in_graveyard(P0, "Whispers of the Muse"));
    assert!(!t.in_hand(P0, "Whispers of the Muse"));
}

#[test]
fn a_buyback_spell_whose_targets_are_all_illegal_goes_to_the_graveyard() {
    cr!("702.27a");
    assert_supported("Capsize");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Capsize");
    t.cast(P0, c).target(bears).kicked(true).go();
    // The target leaves before Capsize resolves.
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Capsize"));
}

#[test]
fn a_buyback_spell_can_be_cast_again_and_again() {
    cr!("702.27a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 12);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let c = t.hand(P0, "Capsize");
    t.cast(P0, c).target(a).kicked(true).go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    let c = t.g.find_in_zone(object::Zone::Hand(P0), "Capsize")[0];
    t.cast(P0, c).target(b).kicked(true).go();
    t.resolve();
    assert!(t.in_hand(P1, "Llanowar Elves"));
    assert!(t.in_hand(P0, "Capsize"));
}

#[test]
fn effects_can_reduce_the_generic_part_of_a_buyback_cost() {
    cr!("702.27a", "601.2f");
    ruling!("Memory Crystal", "Only affects generic mana portions of Buyback costs.");
    ruling!("Memory Crystal", "It applies to all players.");
    assert_supported("Memory Crystal");
    assert_supported("Mind Peel");
    // Whispers of the Muse: {U} + buyback {5} - {2} = four mana.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Memory Crystal");
    t.lands(P0, "Island", 4);
    let w = t.hand(P0, "Whispers of the Muse");
    t.cast(P0, w).kicked(true).go();
    assert_eq!(optional_costs_offered(&t, P0), vec!["buyback".to_string()]);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P0, "Whispers of the Muse"));
    // Mind Peel: {B} + buyback {2}{B}{B} - {2} = {B}{B}{B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Memory Crystal");
    t.lands(P0, "Swamp", 3);
    let m = t.hand(P0, "Mind Peel");
    t.hand(P1, "Grizzly Bears");
    t.cast(P0, m).target(P1).kicked(true).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P0, "Mind Peel"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn buyback_cost_reductions_dont_reduce_the_rest_of_the_cost() {
    cr!("702.27a", "601.2f");
    ruling!("Memory Crystal", "Can’t reduce the cost below zero.");
    ruling!("Memory Crystal", "Does not apply to other parts of the cost.");
    // Two Memory Crystals reduce Capsize's buyback {3} to {0}, but not its {1}{U}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Memory Crystal");
    t.battlefield(P0, "Memory Crystal");
    t.lands(P0, "Island", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Capsize");
    t.cast(P0, c).target(bears).kicked(true).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P0, "Capsize"));
}
