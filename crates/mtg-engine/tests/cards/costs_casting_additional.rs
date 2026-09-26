//! Choices between additional costs (CR 601.2b, 601.2f): "As an additional cost to cast
//! this spell, sacrifice a creature or pay {3}{B}.", "..., sacrifice a creature, discard
//! a card, or pay 4 life.", "... reveal a Merfolk card from your hand or pay {3}." And
//! "If you control a commander, you may cast this spell without paying its mana cost."

use mtg_engine::decision::Answer;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn tapped(t: &TestGame, lands: &[ObjectId]) -> usize {
    lands.iter().filter(|l| t.obj_now(**l).tapped).count()
}

#[test]
fn sacrifice_a_creature_or_pay_mana() {
    cr!("601.2b", "601.2f");
    ruling!(
        "Spark Harvest",
        "You must sacrifice exactly one creature or pay an extra {3}{B} to cast this spell"
    );
    compiles("Spark Harvest");
    // Sacrificing: only {B}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let target = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    let harvest = t.hand(P0, "Spark Harvest");
    let lands = t.lands(P0, "Swamp", 5);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, harvest).target(target).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tapped(&t, &lands), 1);
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));

    // Paying: {3}{B}{B}, and the creature stays.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let target = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let harvest = t.hand(P0, "Spark Harvest");
    let lands = t.lands(P0, "Swamp", 5);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, harvest).target(target).go();
    assert!(t.on_battlefield(bears));
    assert_eq!(tapped(&t, &lands), 5);

    // Without a creature, only paying is possible; without the mana either, it can't be
    // cast.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let target = t.battlefield(P1, "Hill Giant");
    let harvest = t.hand(P0, "Spark Harvest");
    t.lands(P0, "Swamp", 4);
    assert!(t.cast(P0, harvest).target(target).try_go().is_err());
    assert!(t.in_hand(P0, "Spark Harvest"));
}

#[test]
fn three_additional_costs_to_choose_from() {
    cr!("601.2b");
    compiles("Dusk Mangler");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mangler = t.hand(P0, "Dusk Mangler");
    t.hand(P0, "Island");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 7);
    // Pay 4 life.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.cast(P0, mangler).go();
    assert_eq!(t.life(P0), 16);
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Discard a card.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mangler = t.hand(P0, "Dusk Mangler");
    t.hand(P0, "Island");
    t.lands(P0, "Swamp", 7);
    // No creature: the options are discarding and paying life.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, mangler).go();
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn reveal_a_card_or_pay_mana() {
    cr!("601.2b", "701.20a");
    compiles("Silvergill Adept");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let adept = t.hand(P0, "Silvergill Adept");
    let lands = t.lands(P0, "Island", 5);
    // The Adept can't reveal itself: with no other Merfolk, {3} more.
    t.cast(P0, adept).go();
    assert_eq!(tapped(&t, &lands), 5);
    t.resolve();

    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let adept = t.hand(P0, "Silvergill Adept");
    t.hand(P0, "Silvergill Adept");
    let lands = t.lands(P0, "Island", 5);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, adept).go();
    assert_eq!(tapped(&t, &lands), 2);
    // The revealed card stays in hand.
    assert!(t.in_hand(P0, "Silvergill Adept"));
}

#[test]
fn cast_without_paying_its_mana_cost_if_you_control_a_commander() {
    cr!("118.9", "903.3");
    ruling!(
        "Fierce Guardianship",
        "It doesn't matter whose commander you control. Any one will do."
    );
    compiles("Fierce Guardianship");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let divination = t.hand(P1, "Divination");
    t.lands(P1, "Island", 3);
    let spell = t.cast(P1, divination).go();
    let fg = t.hand(P0, "Fierce Guardianship");
    let island = t.lands(P0, "Island", 1)[0];
    let alts = |t: &TestGame| -> Vec<CastMethod> {
        t.cast_options(P0, fg)
            .into_iter()
            .map(|o| o.method)
            .filter(|m| matches!(m, CastMethod::Alternative(_)))
            .collect()
    };
    // A non-commander creature doesn't enable it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(alts(&t).is_empty());
    // An opponent's commander that P0 controls does.
    t.g.objects[bears.0 as usize].is_commander = true;
    t.g.objects[bears.0 as usize].owner = P1;
    let a = alts(&t);
    assert_eq!(a.len(), 1);
    t.cast(P0, fg).method(a[0].clone()).target(spell).go();
    assert!(!t.obj_now(island).tapped);
    t.resolve();
    assert!(t.in_graveyard(P1, "Divination"));
}
