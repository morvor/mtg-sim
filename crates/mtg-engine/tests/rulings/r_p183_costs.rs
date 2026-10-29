//! Rulings batch P183 — typal cost reductions: the Bannerets (one reduction for a spell
//! with both types, generic mana only, on alternative costs; CR 118.7a, 601.2f) and
//! Morophon (colored mana, after additional and alternative costs, hybrid halves; CR
//! 118.7c, 118.7e, 601.2f).

use crate::r_p194_common::{assert_total_cost, casts_for, mana, pool};
use crate::r_s01_common::supported;
use crate::r_s24_common::choose_creature_type;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn with(card: &'static str) -> impl Fn(&mut TestGame) {
    move |t: &mut TestGame| {
        t.battlefield(P0, card);
    }
}

/// Morophon on P0's battlefield, `ty` chosen.
fn morophon(ty: &'static str) -> impl Fn(&mut TestGame) {
    move |t: &mut TestGame| {
        choose_creature_type(t, P0, ty);
        t.enter(P0, "Morophon, the Boundless");
    }
}

/// Whether P0 can cast `name` from hand with `method` (kicked if `kicked`) for exactly
/// the given mana, all of it spent.
fn casts_with(
    setup: &dyn Fn(&mut TestGame),
    name: &str,
    method: &CastMethod,
    kicked: bool,
    colored: &[(ManaType, u32)],
    generic: u32,
) -> bool {
    let mut t = TestGame::new(2);
    setup(&mut t);
    let c = t.hand(P0, name);
    for (ty, n) in colored {
        mana(&mut t, P0, *ty, *n);
    }
    mana(&mut t, P0, ManaType::C, generic);
    t.cast(P0, c).method(method.clone()).kicked(kicked).try_go().is_ok() && pool(&t, P0) == 0
}

#[test]
fn a_spell_with_both_banneret_types_costs_only_1_less() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Stonybrook Banneret",
        "A spell you cast that's both creature types costs {1} less to cast, not {2} less."
    );
    ruling!(
        "Ballyrush Banneret",
        "Spells you cast that have both types cost {1} less to cast, not {2} less."
    );
    supported("Stonybrook Banneret");
    supported("Ballyrush Banneret");
    // Merfolk Seer ({2}{U}, a Merfolk Wizard) and Burrenton Bombardier ({2}{W}, a Kithkin
    // Soldier).
    assert_total_cost(
        &with("Stonybrook Banneret"),
        P0,
        "Merfolk Seer",
        &[(ManaType::U, 1)],
        1,
    );
    assert_total_cost(
        &with("Ballyrush Banneret"),
        P0,
        "Burrenton Bombardier",
        &[(ManaType::W, 1)],
        1,
    );
}

#[test]
fn ballyrush_banneret_reduces_only_generic_mana_and_not_the_mana_cost() {
    cr!("118.7a", "601.2f", "202.3");
    ruling!(
        "Ballyrush Banneret",
        "Ballyrush Banneret's ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell."
    );
    ruling!(
        "Ballyrush Banneret",
        "Ballyrush Banneret's ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay to cast Kithkin and Soldier spells."
    );
    // Goldmeadow Harrier ({W}, a Kithkin Soldier) still costs {W}.
    let setup = with("Ballyrush Banneret");
    assert!(casts_for(&setup, P0, "Goldmeadow Harrier", &[(ManaType::W, 1)], 0));
    assert!(!casts_for(&setup, P0, "Goldmeadow Harrier", &[], 0));
    assert!(!casts_for(&setup, P0, "Goldmeadow Harrier", &[], 1));
    // Burrenton Bombardier cast for {1}{W} still has mana cost {2}{W} and mana value 3.
    let mut t = TestGame::new(2);
    setup(&mut t);
    let b = t.hand(P0, "Burrenton Bombardier");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, b).go();
    let o = t.g.obj(spell);
    assert_eq!(format!("{}", o.chars.mana_cost.clone().unwrap()), "{2}{W}");
    assert_eq!(o.chars.mana_value(), 3);
}

#[test]
fn brighthearth_banneret_reduces_an_evoke_cost() {
    cr!("601.2f", "118.7a", "702.74a");
    ruling!(
        "Brighthearth Banneret",
        "When you cast an Elemental spell by paying its evoke cost, this effect reduces the cost to cast that spell by {1}."
    );
    supported("Brighthearth Banneret");
    supported("Mulldrifter");
    let evoke = CastMethod::Keyword(KeywordKind::Evoke);
    let setup = with("Brighthearth Banneret");
    // Mulldrifter's evoke cost {2}{U} becomes {1}{U}.
    assert!(casts_with(&setup, "Mulldrifter", &evoke, false, &[(ManaType::U, 1)], 1));
    assert!(!casts_with(&setup, "Mulldrifter", &evoke, false, &[(ManaType::U, 1)], 0));
}

#[test]
fn morophon_reduces_colored_mana_of_additional_and_alternative_costs() {
    cr!("601.2f", "118.7c", "702.74a");
    ruling!(
        "Morophon, the Boundless",
        "Cost reduction effects are applied after other cost modifiers, so Morophon can reduce additional costs or alternative costs of spells of the chosen type."
    );
    supported("Morophon, the Boundless");
    supported("Phyrexian Warhorse");
    let evoke = CastMethod::Keyword(KeywordKind::Evoke);
    // Mulldrifter's evoke cost {2}{U} becomes {2}.
    let elemental = morophon("Elemental");
    assert!(casts_with(&elemental, "Mulldrifter", &evoke, false, &[], 2));
    assert!(!casts_with(&elemental, "Mulldrifter", &evoke, false, &[], 1));
    // Phyrexian Warhorse ({3}{B}, kicker {W}) kicked: {3}{W}{B} becomes {3}.
    let horse = morophon("Horse");
    let warhorse = |kicked, generic| {
        casts_with(&horse, "Phyrexian Warhorse", &CastMethod::Normal, kicked, &[], generic)
    };
    assert!(warhorse(true, 3));
    assert!(!warhorse(true, 2));
    assert!(warhorse(false, 3));
}

#[test]
fn morophon_reduces_both_hybrid_halves_once_they_are_chosen() {
    cr!("601.2f", "118.7e");
    ruling!(
        "Morophon, the Boundless",
        "If a spell has hybrid mana symbols in its mana cost, you choose which half you will be paying before determining the total cost. For example, if a spell of the chosen type costs {2}{W/U}{W/U}, you may choose for the cost to be {2}{W}{U} and then reduce it to {2}."
    );
    supported("Portal Manipulator");
    // Portal Manipulator: {2}{W/U}{W/U}, a Human Wizard.
    assert_total_cost(&morophon("Wizard"), P0, "Portal Manipulator", &[], 2);
}
