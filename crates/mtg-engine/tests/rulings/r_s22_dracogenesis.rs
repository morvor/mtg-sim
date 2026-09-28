//! Rulings batch S22 — static abilities that let a player cast spells without paying
//! their mana costs: Dracogenesis ("You may cast Dragon spells without paying their mana
//! costs.") and Omniscience ("You may cast spells from your hand without paying their mana
//! costs."). Casting a spell this way is an alternative cost (CR 118.9): no other
//! alternative cost can be chosen, optional additional costs may be paid, mandatory ones
//! must be (CR 118.9a–b), and X is 0 (CR 107.3b). Timing rules still apply.

use crate::r_s01_common::*;
use crate::r_s07_common::cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FREE: CastMethod = CastMethod::Free;

#[test]
fn dracogenesis_free_dragons_pay_additional_costs_but_no_other_alternative_cost() {
    cr!("118.9", "118.9a", "118.9b", "118.8a", "601.2b");
    ruling!(
        "Dracogenesis",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs those must be paid to cast the spell."
    );
    supported("Dracogenesis");
    supported("Verix Bladewing");
    // Verix Bladewing {2}{R}{R}, kicker {3}: "When Verix Bladewing enters, if it was
    // kicked, create Karox Bladewing, a legendary 4/4 red Dragon creature token with
    // flying." Cast without paying its mana cost, kicked: only the kicker is paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dracogenesis");
    t.lands(P0, "Mountain", 3);
    let verix = t.hand(P0, "Verix Bladewing");
    assert!(cast_methods(&mut t, P0, verix).contains(&FREE));
    t.answer(P0, mtg_engine::testing::DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, verix).method(FREE).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Karox Bladewing").len(), 1);
    // Not kicked: nothing is paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dracogenesis");
    t.lands(P0, "Mountain", 3);
    let verix = t.hand(P0, "Verix Bladewing");
    t.answer(P0, mtg_engine::testing::DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, verix).method(FREE).go();
    assert_eq!(tapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Verix Bladewing").len(), 1);
    assert!(t.named_on_battlefield("Karox Bladewing").is_empty());

    // A Dragon with a mandatory additional cost: it must be paid.
    let hungry = || {
        custom_card(
            "Hungry Dragon",
            "Creature — Dragon",
            "{4}{B}",
            Some((5, 5)),
            "As an additional cost to cast this spell, sacrifice a creature.",
        )
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dracogenesis");
    let dragon = t.custom(P0, hungry(), Zone::Hand(P0));
    assert!(t.cast(P0, dragon).method(FREE).try_go().is_err());
    t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, dragon).method(FREE).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // A Dragon with its own alternative cost: that cost isn't combined with casting it
    // for free — they're two separate ways to cast it.
    let evoker = custom_card(
        "Evoking Dragon",
        "Creature — Dragon",
        "{5}{R}",
        Some((5, 5)),
        "Evoke {2}{R}",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dracogenesis");
    t.lands(P0, "Mountain", 3);
    let dragon = t.custom(P0, evoker, Zone::Hand(P0));
    let methods = cast_methods(&mut t, P0, dragon);
    assert!(methods.contains(&FREE));
    assert!(methods.len() >= 2);
    t.cast(P0, dragon).method(FREE).go();
    assert_eq!(tapped_lands(&t, P0), 0);
    t.resolve_all();
    // Cast for free, not evoked: it stays on the battlefield.
    assert_eq!(t.named_on_battlefield("Evoking Dragon").len(), 1);
    // Not a Dragon: no free way to cast it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dracogenesis");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!cast_methods(&mut t, P0, bears).contains(&FREE));
}

#[test]
fn omniscience_spells_from_hand_are_free_with_x_zero_and_normal_timing() {
    cr!("118.9", "107.3b", "601.3", "307.1");
    supported("Omniscience");
    // "You may cast spells from your hand without paying their mana costs."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).method(FREE).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Blaze with X: X is 0.
    let blaze = t.hand(P0, "Blaze");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, mtg_engine::testing::DecisionKind::X, Answer::Number(5));
    let spell = t.cast(P0, blaze).method(FREE).target(Entity::Object(bears)).go();
    let si = t.obj(spell).stack.as_ref().expect("a spell");
    assert_eq!(si.cast.x.unwrap_or(0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Only from the hand; and a creature only at sorcery speed.
    let gy = t.graveyard(P0, "Hill Giant");
    assert!(!cast_methods(&mut t, P0, gy).contains(&FREE));
    let giant = t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!cast_methods(&mut t, P0, giant).contains(&FREE));
}
