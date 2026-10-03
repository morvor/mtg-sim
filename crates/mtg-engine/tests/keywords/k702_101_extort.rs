//! CR 702.101 Extort.

use crate::common_k702_011_017::{assert_supported, bf, custom_card};
use crate::common_k702_018_026::triggers_on_stack;
use crate::common_k702_027_037::{pay_questions, untapped_lands};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn extort_drains_each_opponent_when_its_cost_is_paid() {
    cr!("702.101", "702.101a");
    ruling!(
        "Syndic of Tithes",
        "You decide whether to pay when the ability resolves."
    );
    assert_supported("Syndic of Tithes");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndic of Tithes");
    t.lands(P0, "Plains", 2);
    // A creature spell triggers it too: "Whenever you cast a spell".
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Extort"), 1);
    // Nothing was paid yet.
    assert_eq!(pay_questions(&t, P0), 0);
    assert_eq!(untapped_lands(&t, P0), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(pay_questions(&t, P0), 1);
    assert_eq!(untapped_lands(&t, P0), 1);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn extort_does_nothing_if_its_cost_isnt_paid() {
    cr!("702.101a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndic of Tithes");
    let plains = t.lands(P0, "Plains", 1);
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(untapped_lands(&t, P0), 1);
    // Without mana to pay, it can't be paid.
    let memnite = t.hand(P0, "Memnite");
    t.g.objects[plains[0].0 as usize].tapped = true;
    t.cast(P0, memnite).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // An opponent's spell doesn't trigger it.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Extort"), 0);
}

#[test]
fn extort_gains_the_total_life_lost_by_all_opponents() {
    cr!("702.101a");
    ruling!(
        "Tithe Drinker",
        "if your opponent's life total can't change (perhaps because that player controls Platinum Emperion), you won't gain any life."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Blind Obedience");
    t.lands(P0, "Swamp", 2);
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    assert_eq!(t.life(P0), 22);
    // P1's life total can't change: only P2 loses life.
    t.battlefield(P1, "Platinum Emperion");
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (19, 18));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn extort_resolves_before_the_spell_even_if_it_is_countered() {
    cr!("702.101a");
    ruling!(
        "Blind Obedience",
        "The extort ability resolves before the spell that caused it to trigger. The ability resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndic of Tithes");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Island", 2);
    let memnite = t.hand(P0, "Memnite");
    let spell = t.cast(P0, memnite).go();
    t.settle();
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(spell).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Memnite"));
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn each_instance_of_extort_triggers_and_is_paid_separately() {
    cr!("702.101b");
    ruling!(
        "Crypt Ghast",
        "You may pay {W/B} a maximum of one time for each extort triggered ability."
    );
    let def = custom_card(
        "Twice-Taxing Cleric",
        "Creature — Human Cleric",
        Some((1, 1)),
        "Extort\nExtort",
    );
    let mut t = TestGame::new(2);
    bf(&mut t, P0, def);
    t.lands(P0, "Plains", 2);
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Extort"), 2);
    // Pay for one of them only.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(pay_questions(&t, P0), 2);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    // Paying for both.
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}
