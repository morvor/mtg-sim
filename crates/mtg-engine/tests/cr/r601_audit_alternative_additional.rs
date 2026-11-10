//! CR 118.8–118.9 / 601.2b audit: alternative and additional costs with real cards. Only
//! one alternative cost (or alternative method of casting) applies to a spell (CR 118.9a,
//! 601.2b), any number of additional costs do (CR 118.8a), additional costs and cost
//! changes apply to an alternative cost (CR 118.9d), and an optional cost that can't be
//! paid can't be chosen (CR 118.3).

use crate::r601_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::offered_costs::OFFERED_ALT_COST;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const OFFERED: CastMethod = CastMethod::Alternative(OFFERED_ALT_COST);
const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const EVOKE: CastMethod = CastMethod::Keyword(KeywordKind::Evoke);
const FACE_DOWN: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);

#[test]
fn kicker_is_paid_with_a_spell_cast_without_paying_its_mana_cost_and_increases_apply() {
    cr!("118.8", "118.9", "601.2b", "601.2f");
    ruling!(
        "Omniscience",
        "You can, however, pay additional costs, such as kicker costs."
    );
    // Burst Lightning kicked under Omniscience: only the kicker's {4} is paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P0, "Mountain", 5);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst)
        .method(CastMethod::Free)
        .kicked(true)
        .target(P1)
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    // Under Thalia too: {4} + {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 5);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst)
        .method(CastMethod::Free)
        .kicked(true)
        .target(P1)
        .go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_free_cast_with_an_increase_and_a_reduction_costs_nothing() {
    cr!("601.2f", "118.9d", "118.5");
    // Omniscience's {0}, plus Thalia's {1}, minus Helm of Awakening's {1}: {0}, paid with
    // no lands at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.battlefield(P0, "Helm of Awakening");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Free));
    t.cast(P0, bolt).method(CastMethod::Free).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Without the Helm the {1} must be paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Free));
    assert!(t.cast(P0, bolt).method(CastMethod::Free).target(P1).try_go().is_err());
}

#[test]
fn only_one_alternative_cost_applies_to_a_spell() {
    cr!("118.9a", "601.2b");
    ruling!(
        "Jodah, Archmage Eternal",
        "You can't combine this with other alternative costs, such as flashback."
    );
    ruling!(
        "Omniscience",
        "you can't choose to cast it for any alternative costs"
    );
    supported("Jodah, Archmage Eternal");
    supported("Mulldrifter");
    // Mulldrifter with Jodah: normally, for its evoke cost, or for Jodah's {W}{U}{B}{R}{G},
    // never for evoke and Jodah's cost together.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jodah, Archmage Eternal");
    let drifter = t.hand(P0, "Mulldrifter");
    let ways = options(&mut t, P0, drifter);
    assert!(ways.iter().any(|o| o.method == CastMethod::Normal && o.alt_cost.is_none()));
    assert!(ways.iter().any(|o| o.method == EVOKE && o.alt_source.is_none()));
    let offered: Vec<_> = ways.iter().filter(|o| o.alt_source.is_some()).collect();
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].method, OFFERED);
    assert!(!ways.iter().any(|o| o.method == EVOKE && o.alt_source.is_some()));
    // Flashback is an alternative cost: Omniscience ("from your hand") doesn't apply to
    // the card in the graveyard, and the flashback cost can't be waived.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let analysis = t.graveyard(P0, "Deep Analysis");
    assert!(!can_cast(&mut t, P0, analysis, FLASHBACK));
    assert!(!can_cast(&mut t, P0, analysis, CastMethod::Free));
    assert!(t.cast(P0, analysis).method(FLASHBACK).target(P0).try_go().is_err());
    assert!(t.in_graveyard(P0, "Deep Analysis"));
    // From the hand it may be cast without paying its mana cost.
    let analysis = t.hand(P0, "Deep Analysis");
    assert!(can_cast(&mut t, P0, analysis, CastMethod::Free));
}

#[test]
fn casting_face_down_is_an_alternative_cost_that_cant_be_waived() {
    cr!("118.9a", "601.2b", "702.37a");
    supported("Willbender");
    // Willbender (morph {1}{U}) under Omniscience: face up for nothing, or face down for
    // {3}; never face down for nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let bender = t.hand(P0, "Willbender");
    assert!(can_cast(&mut t, P0, bender, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, bender, FACE_DOWN));
    assert!(t.cast(P0, bender).method(FACE_DOWN).try_go().is_err());
    assert!(t.in_hand(P0, "Willbender"));
    t.lands(P0, "Island", 3);
    let bender = t.g.current(bender);
    assert!(can_cast(&mut t, P0, bender, FACE_DOWN));
    let spell = t.cast(P0, bender).method(FACE_DOWN).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.obj(spell).face_down);
}

#[test]
fn buyback_is_an_additional_cost_paid_with_an_alternative_cost_and_reduced_after_increases() {
    cr!("118.8", "118.9d", "601.2f", "702.27a");
    ruling!(
        "Memory Crystal",
        "Only affects generic mana portions of Buyback costs."
    );
    supported("Capsize");
    supported("Memory Crystal");
    // Capsize ({1}{U}{U}, buyback {3}) under Omniscience: only the {3} is paid, and the
    // card returns to its owner's hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let capsize = t.hand(P0, "Capsize");
    t.cast(P0, capsize)
        .method(CastMethod::Free)
        .kicked(true)
        .target(bears)
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Capsize"));
    // Two Memory Crystals ({4} less off buyback costs) under Thalia: {1}{U}{U} + {3} + {1}
    // - {4} = {1}{U}{U}. Reducing the buyback cost before the increase would leave
    // {2}{U}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Memory Crystal");
    t.battlefield(P0, "Memory Crystal");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 1);
    let capsize = t.hand(P0, "Capsize");
    assert_eq!(optional_costs_asked(&t, P0).len(), 0);
    t.cast(P0, capsize).kicked(true).target(bears).go();
    assert_eq!(optional_costs_asked(&t, P0), vec!["buyback".to_string()]);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P0, "Capsize"));
}

#[test]
fn an_optional_additional_cost_that_cant_be_paid_isnt_offered() {
    cr!("601.2b", "118.3", "118.8b");
    // Burst Lightning with one Mountain: kicker {4} can't be paid, so it isn't offered,
    // and the spell deals 2.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst).kicked(true).target(P1).go();
    assert!(optional_costs_asked(&t, P0).is_empty());
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // With five Mountains it's offered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst).kicked(false).target(P1).go();
    assert_eq!(optional_costs_asked(&t, P0), vec!["kicker".to_string()]);
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn both_kicker_costs_of_a_spell_may_be_paid() {
    cr!("118.8a", "702.33b", "702.33f", "601.2b");
    supported("Sunscape Battlemage");
    // Sunscape Battlemage {2}{W}, kicker {1}{G} and/or {2}{U}: both kicked, it costs
    // {5}{W}{G}{U}, destroys a flyer and draws two cards.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 5);
    let mage = t.hand(P0, "Sunscape Battlemage");
    let hand = t.hand_size(P0);
    t.cast(P0, mage).kicked(true).kicked(true).go();
    assert_eq!(optional_costs_asked(&t, P0).len(), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.answer_targets(P0, &[Entity::Object(drake)]);
    t.resolve_all();
    assert!(t.on_battlefield(mage));
    assert!(t.in_graveyard(P1, "Wind Drake"));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    // Only the {2}{U} kicker (both are offered, the {1}{G} one declined): it costs
    // {4}{W}{U}, one land stays untapped, and the flyer lives.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 4);
    let mage = t.hand(P0, "Sunscape Battlemage");
    let hand = t.hand_size(P0);
    t.cast(P0, mage).kicked(false).kicked(true).go();
    assert_eq!(optional_costs_asked(&t, P0).len(), 2);
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve_all();
    assert!(t.on_battlefield(drake));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}
